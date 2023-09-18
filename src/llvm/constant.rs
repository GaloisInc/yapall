// SPDX-License-Identifier: BSD-3-Clause
use std::{collections::HashMap, fmt::Display};

use crate::arc::{Arc, UArc};

use super::error::Error;
use super::name::{FunctionName, GlobalName};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct BitCast {
    pub(crate) pointer: Arc<Constant>,
}

impl BitCast {
    pub fn from_bitcast(
        globals: &HashMap<&str, Arc<Constant>>,
        b: &llvm_ir::constant::BitCast,
    ) -> Result<Self, Error> {
        Constant::create(globals, &b.operand).map(|c| BitCast { pointer: c })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct GetElementPtr {
    pub(crate) pointer: Arc<Constant>,
}

impl GetElementPtr {
    pub fn from_getelementptr(
        globals: &HashMap<&str, Arc<Constant>>,
        gep: &llvm_ir::constant::GetElementPtr,
    ) -> Result<Self, Error> {
        Constant::create(globals, &gep.address).map(|c| GetElementPtr { pointer: c })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct IntToPtr {
    int: Arc<Constant>,
}

impl IntToPtr {
    pub fn from_inttoptr(
        globals: &HashMap<&str, Arc<Constant>>,
        int: &llvm_ir::constant::IntToPtr,
    ) -> Result<Self, Error> {
        Constant::create(globals, &int.operand).map(|c| IntToPtr { int: c })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PtrToInt {
    pointer: Arc<Constant>,
}

impl PtrToInt {
    pub fn from_ptrtoint(
        globals: &HashMap<&str, Arc<Constant>>,
        ptr: &llvm_ir::constant::PtrToInt,
    ) -> Result<Self, Error> {
        Constant::create(globals, &ptr.operand).map(|c| PtrToInt { pointer: c })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Struct {
    fields: Vec<Arc<Constant>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Constant {
    Function(UArc<FunctionName>),
    Global(Arc<GlobalName>),
    Int { bits: u32, value: u64 },
    Null,
    // Expressions
    Add,
    Sub,
    Mul,
    UDiv,
    SDiv,
    URem,
    SRem,
    And,
    Or,
    Xor,
    Shl,
    LShr,
    AShr,
    FAdd,
    FSub,
    FMul,
    FDiv,
    FRem,
    ExtractElement,
    InsertElement,
    ShuffleVector,
    ExtractValue,
    InsertValue,
    GetElementPtr(GetElementPtr),
    Trunc,
    ZExt,
    SExt,
    FPTrunc,
    FPExt,
    FPToUI,
    FPToSI,
    UIToFP,
    SIToFP,
    PtrToInt(PtrToInt),
    IntToPtr(IntToPtr),
    BitCast(BitCast),
    AddrSpaceCast,
    ICmp,
    FCmp,
    Select,
    //
    Array(Vec<Arc<Constant>>),
    Struct(Struct),
    Undef,
    //
    Other,
}

impl Constant {
    pub fn create(
        globals: &HashMap<&str, Arc<Constant>>,
        constant: &llvm_ir::Constant,
    ) -> Result<Arc<Self>, Error> {
        match &constant {
            llvm_ir::Constant::GlobalReference { name, .. } => {
                match globals.get::<str>(name.as_ref()) {
                    Some(nm) => Ok(nm.clone()),
                    None => panic!("Couldn't find global {}", name),
                }
            }
            llvm_ir::Constant::Int { value, bits } => Ok(Arc::new(Constant::Int {
                value: *value,
                bits: *bits,
            })),
            llvm_ir::Constant::Null(_) => Ok(Arc::new(Constant::Null)),

            // Expressions
            llvm_ir::Constant::Add(_) => Ok(Arc::new(Constant::Add)),
            llvm_ir::Constant::Sub(_) => Ok(Arc::new(Constant::Sub)),
            llvm_ir::Constant::Mul(_) => Ok(Arc::new(Constant::Mul)),
            llvm_ir::Constant::UDiv(_) => Ok(Arc::new(Constant::UDiv)),
            llvm_ir::Constant::SDiv(_) => Ok(Arc::new(Constant::SDiv)),
            llvm_ir::Constant::URem(_) => Ok(Arc::new(Constant::URem)),
            llvm_ir::Constant::SRem(_) => Ok(Arc::new(Constant::SRem)),
            llvm_ir::Constant::And(_) => Ok(Arc::new(Constant::And)),
            llvm_ir::Constant::Or(_) => Ok(Arc::new(Constant::Or)),
            llvm_ir::Constant::Xor(_) => Ok(Arc::new(Constant::Xor)),
            llvm_ir::Constant::Shl(_) => Ok(Arc::new(Constant::Shl)),
            llvm_ir::Constant::LShr(_) => Ok(Arc::new(Constant::LShr)),
            llvm_ir::Constant::AShr(_) => Ok(Arc::new(Constant::AShr)),
            llvm_ir::Constant::FAdd(_) => Ok(Arc::new(Constant::FAdd)),
            llvm_ir::Constant::FSub(_) => Ok(Arc::new(Constant::FSub)),
            llvm_ir::Constant::FMul(_) => Ok(Arc::new(Constant::FMul)),
            llvm_ir::Constant::FDiv(_) => Ok(Arc::new(Constant::FDiv)),
            llvm_ir::Constant::FRem(_) => Ok(Arc::new(Constant::FRem)),
            llvm_ir::Constant::ExtractElement(_) => Ok(Arc::new(Constant::ExtractElement)),
            llvm_ir::Constant::InsertElement(_) => Ok(Arc::new(Constant::InsertElement)),
            llvm_ir::Constant::ShuffleVector(_) => Ok(Arc::new(Constant::ShuffleVector)),
            llvm_ir::Constant::ExtractValue(_) => Ok(Arc::new(Constant::ExtractValue)),
            llvm_ir::Constant::InsertValue(_) => Ok(Arc::new(Constant::InsertValue)),
            llvm_ir::Constant::GetElementPtr(g) => GetElementPtr::from_getelementptr(globals, g)
                .map(Constant::GetElementPtr)
                .map(Arc::new),
            llvm_ir::Constant::Trunc(_) => Ok(Arc::new(Constant::Trunc)),
            llvm_ir::Constant::ZExt(_) => Ok(Arc::new(Constant::ZExt)),
            llvm_ir::Constant::SExt(_) => Ok(Arc::new(Constant::SExt)),
            llvm_ir::Constant::FPTrunc(_) => Ok(Arc::new(Constant::FPTrunc)),
            llvm_ir::Constant::FPExt(_) => Ok(Arc::new(Constant::FPExt)),
            llvm_ir::Constant::FPToUI(_) => Ok(Arc::new(Constant::FPToUI)),
            llvm_ir::Constant::FPToSI(_) => Ok(Arc::new(Constant::FPToSI)),
            llvm_ir::Constant::UIToFP(_) => Ok(Arc::new(Constant::UIToFP)),
            llvm_ir::Constant::SIToFP(_) => Ok(Arc::new(Constant::SIToFP)),
            llvm_ir::Constant::PtrToInt(c) => Ok(Arc::new(Constant::PtrToInt(
                PtrToInt::from_ptrtoint(globals, c)?,
            ))),
            llvm_ir::Constant::IntToPtr(c) => Ok(Arc::new(Constant::IntToPtr(
                IntToPtr::from_inttoptr(globals, c)?,
            ))),
            llvm_ir::Constant::BitCast(b) => BitCast::from_bitcast(globals, b)
                .map(Constant::BitCast)
                .map(Arc::new),
            llvm_ir::Constant::AddrSpaceCast(_) => Ok(Arc::new(Constant::AddrSpaceCast)),
            llvm_ir::Constant::ICmp(_) => Ok(Arc::new(Constant::ICmp)),
            llvm_ir::Constant::FCmp(_) => Ok(Arc::new(Constant::FCmp)),
            llvm_ir::Constant::Select(_) => Ok(Arc::new(Constant::Select)),
            //
            llvm_ir::Constant::Array { elements, .. } => {
                let mut es = Vec::with_capacity(elements.len());
                for e in elements {
                    es.push(Constant::create(globals, e)?);
                }
                Ok(Arc::new(Constant::Array(es)))
            }
            llvm_ir::Constant::Struct { values, .. } => {
                let mut fields = Vec::with_capacity(values.len());
                for v in values {
                    fields.push(Constant::create(globals, v)?);
                }
                Ok(Arc::new(Constant::Struct(Struct { fields })))
            }
            llvm_ir::Constant::Undef(_) => Ok(Arc::new(Constant::Undef)),
            _ => Ok(Arc::new(Constant::Other)),
            // c => panic!("Unhandled constant: {c}")
            // llvm_ir::Constant::Float(_) => todo!(),
            // llvm_ir::Constant::AggregateZero(_) => todo!(),
            // llvm_ir::Constant::Vector(_) => todo!(),
            // llvm_ir::Constant::Poison(_) => todo!(),
            // llvm_ir::Constant::BlockAddress => todo!(),
            // llvm_ir::Constant::TokenNone => todo!(),
        }
    }

    pub fn pointers(&self) -> Vec<Constant> {
        match self {
            Constant::Function(_) => vec![self.clone()],
            Constant::Global(_) => vec![self.clone()],
            Constant::Int { .. } => vec![self.clone()],
            Constant::Null => vec![self.clone()],
            Constant::Add => vec![],            // TODO
            Constant::Sub => vec![],            // TODO
            Constant::Mul => vec![],            // TODO
            Constant::UDiv => vec![],           // TODO
            Constant::SDiv => vec![],           // TODO
            Constant::URem => vec![],           // TODO
            Constant::SRem => vec![],           // TODO
            Constant::And => vec![],            // TODO
            Constant::Or => vec![],             // TODO
            Constant::Xor => vec![],            // TODO
            Constant::Shl => vec![],            // TODO
            Constant::LShr => vec![],           // TODO
            Constant::AShr => vec![],           // TODO
            Constant::FAdd => vec![],           // TODO
            Constant::FSub => vec![],           // TODO
            Constant::FMul => vec![],           // TODO
            Constant::FDiv => vec![],           // TODO
            Constant::FRem => vec![],           // TODO
            Constant::ExtractElement => vec![], // TODO
            Constant::InsertElement => vec![],  // TODO
            Constant::ShuffleVector => vec![],  // TODO
            Constant::ExtractValue => vec![],   // TODO
            Constant::InsertValue => vec![],    // TODO
            Constant::GetElementPtr(g) => g.pointer.pointers(),
            Constant::Trunc => vec![],   // TODO
            Constant::ZExt => vec![],    // TODO
            Constant::SExt => vec![],    // TODO
            Constant::FPTrunc => vec![], // TODO
            Constant::FPExt => vec![],   // TODO
            Constant::FPToUI => vec![],  // TODO
            Constant::FPToSI => vec![],  // TODO
            Constant::UIToFP => vec![],  // TODO
            Constant::SIToFP => vec![],  // TODO
            Constant::PtrToInt(c) => c.pointer.pointers(),
            Constant::IntToPtr(c) => c.int.pointers(),
            Constant::BitCast(b) => b.pointer.pointers(),
            Constant::AddrSpaceCast => vec![], // TODO
            Constant::ICmp => vec![],          // TODO
            Constant::FCmp => vec![],          // TODO
            Constant::Select => vec![],
            //
            Constant::Array(v) => v.iter().flat_map(|c| c.pointers()).collect(),
            Constant::Struct(Struct { fields }) => {
                fields.iter().flat_map(|c| c.pointers()).collect()
            }
            Constant::Undef => vec![self.clone()],
            //
            Constant::Other => vec![],
        }
    }
}

impl Display for Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Constant::Function(f) => format!("{}", f),
                Constant::Global(g) => format!("{}", g),
                Constant::Int { value, bits } => format!("i{} {}", bits, value),
                Constant::Null => "null".to_string(),
                // Expressions
                Constant::Add => "Add".to_string(),
                Constant::Sub => "Sub".to_string(),
                Constant::Mul => "Mul".to_string(),
                Constant::UDiv => "UDiv".to_string(),
                Constant::SDiv => "SDiv".to_string(),
                Constant::URem => "URem".to_string(),
                Constant::SRem => "SRem".to_string(),
                Constant::And => "And".to_string(),
                Constant::Or => "Or".to_string(),
                Constant::Xor => "Xor".to_string(),
                Constant::Shl => "Shl".to_string(),
                Constant::LShr => "LShr".to_string(),
                Constant::AShr => "AShr".to_string(),
                Constant::FAdd => "FAdd".to_string(),
                Constant::FSub => "FSub".to_string(),
                Constant::FMul => "FMul".to_string(),
                Constant::FDiv => "FDiv".to_string(),
                Constant::FRem => "FRem".to_string(),
                Constant::ExtractElement => "ExtractElement".to_string(),
                Constant::InsertElement => "InsertElement".to_string(),
                Constant::ShuffleVector => "ShuffleVector".to_string(),
                Constant::ExtractValue => "ExtractValue".to_string(),
                Constant::InsertValue => "InsertValue".to_string(),
                Constant::GetElementPtr(g) => format!("getelementptr({})", g.pointer),
                Constant::Trunc => "Trunc".to_string(),
                Constant::ZExt => "ZExt".to_string(),
                Constant::SExt => "SExt".to_string(),
                Constant::FPTrunc => "FPTrunc".to_string(),
                Constant::FPExt => "FPExt".to_string(),
                Constant::FPToUI => "FPToUI".to_string(),
                Constant::FPToSI => "FPToSI".to_string(),
                Constant::UIToFP => "UIToFP".to_string(),
                Constant::SIToFP => "SIToFP".to_string(),
                Constant::PtrToInt(c) => format!("ptrtoint({})", c.pointer),
                Constant::IntToPtr(c) => format!("inttoptr({})", c.int),
                Constant::BitCast(b) => format!("bitcast({})", b.pointer),
                Constant::AddrSpaceCast => "AddrSpaceCast".to_string(),
                Constant::ICmp => "ICmp".to_string(),
                Constant::FCmp => "FCmp".to_string(),
                Constant::Select => "Select".to_string(),
                //
                Constant::Array(a) => format!(
                    "[ {} ]",
                    a.iter()
                        .map(|c| format!("{}", c))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Constant::Struct(s) => format!(
                    "{{ {} }}",
                    s.fields
                        .iter()
                        .map(|c| format!("{}", c))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Constant::Undef => "undef".to_string(),
                //
                Constant::Other => "<some constant>".to_string(),
            }
        )
    }
}
