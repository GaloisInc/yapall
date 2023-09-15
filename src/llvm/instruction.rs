// SPDX-License-Identifier:i BSD-3-Clause
use std::collections::HashMap;

use either::Either;
use llvm_ir::{module::DataLayout, types::Types, Name};

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
}

impl Alloca {
    pub(crate) fn from_alloca<'module>(
        _dl: &'module DataLayout,
        _types: &'module Types,
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        alloca: &'module llvm_ir::instruction::Alloca,
    ) -> Result<Self, Error> {
        Ok(Alloca {
            num_elements: Operand::new(operands, globals, locals, &alloca.num_elements)?,
        })
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
    pub(crate) pointer: Arc<Operand>,
}

impl GetElementPtr {
    pub(crate) fn from_gep<'module>(
        _dl: &DataLayout,
        _types: &Types,
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        gep: &'module llvm_ir::instruction::GetElementPtr,
    ) -> Result<Self, Error> {
        Ok(GetElementPtr {
            pointer: Operand::new(operands, globals, locals, &gep.address)?,
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
            Opcode::GetElementPtr(GetElementPtr { pointer }) => vec![pointer.clone()],
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
