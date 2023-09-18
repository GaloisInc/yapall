// SPDX-License-Identifier: BSD-3-Clause
use std::collections::HashMap;

use llvm_ir::Name;

use crate::arc::{Arc, UArc};

use super::constant::Constant;
use super::error::Error;
use super::name::{InstructionName, LocalName};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Operand {
    Constant(Arc<Constant>),
    Local(Arc<LocalName>),
    Metadata,
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Operand::Constant(c) => format!("{}", c),
                Operand::Local(l) => format!("{}", l),
                Operand::Metadata => "<metadata>".to_string(),
            }
        )
    }
}

impl Operand {
    pub(crate) fn constant_int(&self) -> Option<u64> {
        if let Operand::Constant(const_arc) = &self {
            if let Constant::Int { value, .. } = &**const_arc {
                Some(*value)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn create(
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        op: &llvm_ir::Operand,
    ) -> Result<Arc<Self>, Error> {
        match op {
            // TODO: Share Metadata with lazy_static
            llvm_ir::Operand::MetadataOperand => Ok(Arc::new(Operand::Metadata)),
            // TODO: Share Constant with a locals-like map
            llvm_ir::Operand::ConstantOperand(constant_ref) => {
                Constant::create(globals, constant_ref)
                    .map(Operand::Constant)
                    .map(Arc::new)
            }
            llvm_ir::Operand::LocalOperand { name, .. } => locals
                .get(name)
                .cloned()
                .ok_or_else(|| Error(format!("Bad local: {}\n{:#?}", name, locals))),
        }
    }

    pub(crate) fn new<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        llvm_op: &'module llvm_ir::Operand,
    ) -> Result<Arc<Self>, Error> {
        let op = Self::create(globals, locals, llvm_op);
        if let Ok(o) = &op {
            operands.insert(o.clone(), llvm_op);
        }
        op
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Callee {
    Operand(Arc<Operand>),
    Asm,
}

/// An [`Operand`] that is guaranteed to hold an [`InstructionName`]. There
/// are many places in the LLVM module structure and the pointer analysis where an
/// instruction need be treated as an `Operand`, better to allocate the `Operand`
/// once and share it everywhere with an [`Arc`].
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InstructionOperand(Arc<Operand>);

#[inline]
#[allow(unused_variables)]
fn unreachable(s: &str) -> ! {
    if cfg!(not(release)) {
        unreachable!("{}", s)
    } else {
        unsafe { std::hint::unreachable_unchecked() }
    }
}

impl InstructionOperand {
    pub(crate) fn new(i: UArc<InstructionName>) -> Self {
        InstructionOperand(Arc::new(Operand::Local(Arc::new(LocalName::Instruction(
            i,
        )))))
    }

    #[inline]
    pub(crate) fn instruction_name(&self) -> UArc<InstructionName> {
        match self.0.as_ref() {
            Operand::Local(arc) => match arc.as_ref() {
                LocalName::Instruction(i) => i.clone(),
                _ => unreachable("InstructionOperand invariant broken"),
            },
            _ => unreachable("InstructionOperand invariant broken"),
        }
    }

    #[inline]
    pub(crate) fn _into_operand(self) -> Arc<Operand> {
        self.0
    }

    #[inline]
    pub(crate) fn operand(&self) -> Arc<Operand> {
        self.0.clone()
    }
}
