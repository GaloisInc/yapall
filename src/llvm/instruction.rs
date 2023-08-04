use std::collections::HashMap;

use either::Either;
use llvm_ir::{
    module::{DataLayout, TypeSize},
    types::{Typed, Types},
    Name,
};

use crate::arc::Arc;

use super::constant::Constant;
use super::error::Error;
use super::operand::{Callee, Operand};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Add {
    pub(crate) operand0: Arc<Operand>,
    pub(crate) operand1: Arc<Operand>,
}

impl Add {
    pub(crate) fn from_add<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        i: &'module llvm_ir::instruction::Add,
    ) -> Result<Self, Error> {
        Ok(Add {
            operand0: Operand::new(operands, globals, locals, &i.operand0)?,
            operand1: Operand::new(operands, globals, locals, &i.operand1)?,
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Alloca {
    pub(crate) num_elements: Arc<Operand>,
    /// In bytes
    pub(crate) type_size: TypeSize,
}

impl Alloca {
    pub(crate) fn from_alloca<'module>(
        dl: &'module DataLayout,
        types: &'module Types,
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        alloca: &'module llvm_ir::instruction::Alloca,
    ) -> Result<Self, Error> {
        Ok(Alloca {
            num_elements: Operand::new(operands, globals, locals, &alloca.num_elements)?,
            type_size: dl
                .get_type_alloc_size(types, &alloca.allocated_type)
                .expect("alloca of unsized type?"),
        })
    }

    /// In bytes
    pub(crate) fn size(&self) -> Option<TypeSize> {
        self.num_elements.constant_int().map(|i| self.type_size * i)
    }

    pub(crate) fn min_size(&self) -> Option<u64> {
        self.size().map(|sz| sz.min_size())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BitCast {
    pub(crate) pointer: Arc<Operand>,
}

impl BitCast {
    pub(crate) fn from_bitcast<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        i: &'module llvm_ir::instruction::BitCast,
    ) -> Result<Self, Error> {
        Ok(BitCast {
            pointer: Operand::new(operands, globals, locals, &i.operand)?,
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Call {
    pub callee: Callee,
    pub args: Vec<Arc<Operand>>,
}

impl Call {
    pub(crate) fn from_call<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        call: &'module llvm_ir::instruction::Call,
    ) -> Result<Self, Error> {
        Ok(Call {
            callee: match &call.function {
                Either::Left(_asm) => Callee::Asm,
                Either::Right(op) => Callee::Operand(Operand::new(operands, globals, locals, op)?),
            },
            args: call
                .arguments
                .iter()
                .map(|(op, _)| {
                    Operand::new(operands, globals, locals, op).expect("Malformed LLVM module!")
                })
                .collect(),
        })
    }

    pub(crate) fn operands(&self) -> Vec<Arc<Operand>> {
        let mut os = match &self.callee {
            Callee::Asm => vec![],
            Callee::Operand(op) => vec![op.clone()],
        };
        os.extend(self.args.iter().cloned());
        os
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct GetElementPtr {
    pub(crate) indices: Vec<Arc<Operand>>,
    pub(crate) pointer: Arc<Operand>,
    /// In bytes
    pub(crate) offset: Option<i64>,
}

impl GetElementPtr {
    fn gep_type(
        dl: &DataLayout,
        types: &Types,
        ty: &llvm_ir::Type,
        idx: Option<i64>,
    ) -> (llvm_ir::TypeRef, Option<i64>) {
        match ty {
            llvm_ir::Type::VectorType {
                element_type,
                num_elements,
                scalable,
            } => {
                if *scalable {
                    panic!("GEP of scalable vector type");
                }
                if let Some(i) = idx {
                    let iusize = usize::try_from(i).unwrap();
                    if iusize > *num_elements {
                        panic!("Bad array index in GEP");
                    }
                }
                let elt_sz = dl
                    .get_type_alloc_size(types, element_type)
                    .unwrap()
                    .min_size();
                (
                    element_type.clone(),
                    idx.map(|i| i * i64::try_from(elt_sz).unwrap()),
                )
            }
            llvm_ir::Type::ArrayType {
                element_type,
                num_elements,
            } => {
                if let Some(i) = idx {
                    let iusize = usize::try_from(i).unwrap();
                    if iusize > *num_elements {
                        panic!("Bad array index in GEP");
                    }
                }
                let elt_sz = dl
                    .get_type_alloc_size(types, element_type)
                    .unwrap()
                    .min_size();
                (
                    element_type.clone(),
                    idx.map(|i| i * i64::try_from(elt_sz).unwrap()),
                )
            }

            llvm_ir::Type::StructType {
                element_types,
                is_packed: _,
            } => match idx {
                Some(i) => {
                    let iusize = usize::try_from(i).unwrap();
                    if iusize >= element_types.len() {
                        panic!("Bad struct index in GEP");
                    }
                    // TODO: Need struct layout info from llvm-ir
                    (element_types[iusize].clone(), None)
                }
                None => panic!("GEP of struct with non-const index"),
            },
            llvm_ir::Type::NamedStructType { name } => match types.named_struct_def(name) {
                Some(llvm_ir::types::NamedStructDef::Defined(def)) => {
                    Self::gep_type(dl, types, def, idx)
                }
                _ => panic!("GEP into opaque struct"),
            },
            // No `_` pattern for future-proofing
            llvm_ir::Type::PointerType { .. } => panic!("GEP of non-aggregate"),
            llvm_ir::Type::FuncType { .. } => panic!("GEP of non-aggregate"),
            llvm_ir::Type::FPType(_) => panic!("GEP of non-aggregate"),
            llvm_ir::Type::VoidType => panic!("GEP of non-aggregate"),
            llvm_ir::Type::IntegerType { .. } => panic!("GEP of non-aggregate"),
            llvm_ir::Type::X86_MMXType => panic!("GEP of non-aggregate"),
            llvm_ir::Type::X86_AMXType => panic!("GEP of non-aggregate"),
            llvm_ir::Type::MetadataType => panic!("GEP of non-aggregate"),
            llvm_ir::Type::LabelType => panic!("GEP of non-aggregate"),
            llvm_ir::Type::TokenType => panic!("GEP of non-aggregate"),
        }
    }

    pub(crate) fn from_gep<'module>(
        dl: &DataLayout,
        types: &Types,
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        gep: &'module llvm_ir::instruction::GetElementPtr,
    ) -> Result<Self, Error> {
        let mut indices = Vec::with_capacity(gep.indices.len());

        let address_type = gep.address.get_type(types);
        let top_ty = if let llvm_ir::Type::PointerType {
            pointee_type: ty,
            addr_space: _,
        } = &*address_type
        {
            ty.clone()
        } else {
            panic!(
                "Malformed LLVM module: GEP of non-pointer type: {}",
                address_type
            )
        };

        // The first index is special: it indexes the pointer an an array.
        // This means it adds an offset equal to itself times the size of the
        // overall type being indexed.
        //
        // https://www.llvm.org/docs/GetElementPtr.html
        // https://blog.yossarian.net/2020/09/19/LLVMs-getelementptr-by-example
        debug_assert!(!gep.indices.is_empty());
        let first_offset = {
            let first_index_op = Operand::new(operands, globals, locals, &gep.indices[0])?;
            let first_index_op_int = first_index_op.constant_int();
            indices.push(first_index_op);
            if let Some(i) = first_index_op_int {
                let sz = dl
                    .get_type_alloc_size(types, &top_ty)
                    .expect("Malformed LLVM module: GEP index on non-sized type");
                Some((i as i64) * i64::try_from(sz.min_size()).unwrap())
            } else {
                None
            }
        };
        let mut offset = first_offset;

        // eprintln!();
        // eprintln!("{}", gep);

        let mut ty = top_ty.clone();
        let mut all_zero = offset == Some(0); // for assertions later
        for i in &gep.indices[1..] {
            eprintln!("{:?}", offset);
            let idx_op = Operand::new(operands, globals, locals, i)?;
            let idx_op_int = idx_op.constant_int().map(|u| u as i64);
            all_zero &= idx_op_int == Some(0);
            (ty, offset) = Self::gep_type(dl, types, &ty, idx_op_int);
            indices.push(idx_op);
        }

        // We should have computed the right types
        let result_ty = gep.get_type(types);
        let is_named = matches!(&*ty, llvm_ir::Type::NamedStructType { .. });
        assert!(types.pointer_to(ty) == result_ty || is_named);

        // We should have computed an in-bounds index
        let top_sz = dl.get_type_alloc_size(types, &top_ty).unwrap().min_size();
        if let Some(off) = offset {
            if off > 0 {
                assert!(((off - first_offset.unwrap()) as u64) < top_sz);
            }
        }

        // All zero indices should yield an offset of zero
        assert!(!all_zero || (offset == Some(0) || offset.is_none()));

        Ok(GetElementPtr {
            indices,
            pointer: Operand::new(operands, globals, locals, &gep.address)?,
            offset: None,
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct IntToPtr {
    pub(crate) int: Arc<Operand>,
}

impl IntToPtr {
    pub(crate) fn from_inttoptr<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        i: &'module llvm_ir::instruction::IntToPtr,
    ) -> Result<Self, Error> {
        Ok(IntToPtr {
            int: Operand::new(operands, globals, locals, &i.operand)?,
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Icmp {
    pub(crate) operand0: Arc<Operand>,
    pub(crate) operand1: Arc<Operand>,
}

impl Icmp {
    pub(crate) fn from_icmp<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        i: &'module llvm_ir::instruction::ICmp,
    ) -> Result<Self, Error> {
        Ok(Icmp {
            operand0: Operand::new(operands, globals, locals, &i.operand0)?,
            operand1: Operand::new(operands, globals, locals, &i.operand1)?,
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Load {
    pub(crate) pointer: Arc<Operand>,
}

impl Load {
    pub(crate) fn from_load<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        load: &'module llvm_ir::instruction::Load,
    ) -> Result<Self, Error> {
        Ok(Load {
            pointer: Operand::new(operands, globals, locals, &load.address)?,
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Phi {
    pub(crate) values: Vec<Arc<Operand>>,
}

impl Phi {
    pub(crate) fn from_phi<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        phi: &'module llvm_ir::instruction::Phi,
    ) -> Result<Self, Error> {
        let mut values = Vec::new();
        for (value, _name) in &phi.incoming_values {
            values.push(Operand::new(operands, globals, locals, value)?);
        }
        Ok(Phi { values })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PtrToInt {
    pub(crate) pointer: Arc<Operand>,
}

impl PtrToInt {
    pub(crate) fn from_ptrtoint<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        i: &'module llvm_ir::instruction::PtrToInt,
    ) -> Result<Self, Error> {
        Ok(PtrToInt {
            pointer: Operand::new(operands, globals, locals, &i.operand)?,
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Select {
    pub(crate) true_value: Arc<Operand>,
    pub(crate) false_value: Arc<Operand>,
}

impl Select {
    pub(crate) fn from_select<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        select: &'module llvm_ir::instruction::Select,
    ) -> Result<Self, Error> {
        Ok(Select {
            true_value: Operand::new(operands, globals, locals, &select.true_value)?,
            false_value: Operand::new(operands, globals, locals, &select.false_value)?,
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Store {
    pub(crate) value: Arc<Operand>,
    pub(crate) pointer: Arc<Operand>,
}

impl Store {
    pub(crate) fn from_store<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        store: &'module llvm_ir::instruction::Store,
    ) -> Result<Self, Error> {
        Ok(Store {
            pointer: Operand::new(operands, globals, locals, &store.address)?,
            value: Operand::new(operands, globals, locals, &store.value)?,
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Sub {
    pub(crate) minuend: Arc<Operand>,
    pub(crate) subtrahend: Arc<Operand>,
}

impl Sub {
    pub(crate) fn from_sub<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        i: &'module llvm_ir::instruction::Sub,
    ) -> Result<Self, Error> {
        Ok(Sub {
            minuend: Operand::new(operands, globals, locals, &i.operand0)?,
            subtrahend: Operand::new(operands, globals, locals, &i.operand1)?,
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Opcode {
    Add(Add),
    Alloca(Alloca),
    BitCast(BitCast),
    Call(Call),
    GetElementPtr(GetElementPtr),
    Icmp(Icmp),
    IntToPtr(IntToPtr),
    Load(Load),
    Phi(Phi),
    PtrToInt(PtrToInt),
    Select(Select),
    Store(Store),
    Sub(Sub),
    //
    Other,
}

impl Opcode {
    pub(crate) fn from_instruction<'module>(
        dl: &'module DataLayout,
        types: &'module Types,
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        i: &'module llvm_ir::Instruction,
    ) -> Result<Self, Error> {
        Ok(match i {
            llvm_ir::Instruction::Add(add) => {
                Opcode::Add(Add::from_add(operands, globals, locals, add)?)
            }

            llvm_ir::Instruction::Alloca(alloca) => Opcode::Alloca(Alloca::from_alloca(
                dl, types, operands, globals, locals, alloca,
            )?),

            llvm_ir::Instruction::BitCast(bitcast) => {
                Opcode::BitCast(BitCast::from_bitcast(operands, globals, locals, bitcast)?)
            }

            llvm_ir::Instruction::Call(call) => {
                Opcode::Call(Call::from_call(operands, globals, locals, call)?)
            }

            llvm_ir::Instruction::GetElementPtr(gep) => Opcode::GetElementPtr(
                GetElementPtr::from_gep(dl, types, operands, globals, locals, gep)?,
            ),

            llvm_ir::Instruction::ICmp(icmp) => {
                Opcode::Icmp(Icmp::from_icmp(operands, globals, locals, icmp)?)
            }

            llvm_ir::Instruction::IntToPtr(itp) => {
                Opcode::IntToPtr(IntToPtr::from_inttoptr(operands, globals, locals, itp)?)
            }

            llvm_ir::Instruction::Load(load) => {
                Opcode::Load(Load::from_load(operands, globals, locals, load)?)
            }

            llvm_ir::Instruction::Phi(phi) => {
                Opcode::Phi(Phi::from_phi(operands, globals, locals, phi)?)
            }

            llvm_ir::Instruction::PtrToInt(load) => {
                Opcode::PtrToInt(PtrToInt::from_ptrtoint(operands, globals, locals, load)?)
            }

            llvm_ir::Instruction::Select(select) => {
                Opcode::Select(Select::from_select(operands, globals, locals, select)?)
            }

            llvm_ir::Instruction::Store(store) => {
                Opcode::Store(Store::from_store(operands, globals, locals, store)?)
            }

            llvm_ir::Instruction::Sub(sub) => {
                Opcode::Sub(Sub::from_sub(operands, globals, locals, sub)?)
            }

            _ => Opcode::Other,
        })
    }

    pub(crate) fn operands(&self) -> Vec<Arc<Operand>> {
        match self {
            Opcode::Add(a) => vec![a.operand0.clone(), a.operand1.clone()],
            Opcode::Alloca(Alloca { .. }) => vec![],
            Opcode::BitCast(BitCast { pointer }) => vec![pointer.clone()],
            Opcode::Call(c) => c.operands(),
            Opcode::GetElementPtr(GetElementPtr {
                pointer,
                indices,
                offset: _,
            }) => {
                let mut ops = indices.clone();
                ops.push(pointer.clone());
                ops
            }
            Opcode::Icmp(i) => vec![i.operand0.clone(), i.operand1.clone()],
            Opcode::IntToPtr(i) => vec![i.int.clone()],
            Opcode::Load(l) => vec![l.pointer.clone()],
            Opcode::Phi(i) => i.values.clone(),
            Opcode::PtrToInt(i) => vec![i.pointer.clone()],
            Opcode::Select(s) => vec![s.true_value.clone(), s.false_value.clone()],
            Opcode::Store(s) => vec![s.value.clone(), s.pointer.clone()],
            Opcode::Sub(i) => vec![i.minuend.clone(), i.subtrahend.clone()],
            //
            Opcode::Other => vec![],
        }
    }
}
