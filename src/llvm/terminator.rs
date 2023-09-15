// SPDX-License-Identifier:i BSD-3-Clause
use std::collections::HashMap;

use either::Either;
use llvm_ir::Name;

use crate::arc::Arc;

use super::constant::Constant;
use super::error::Error;
use super::operand::{Callee, Operand};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Invoke {
    pub callee: Callee,
    pub args: Vec<Arc<Operand>>,
}

impl Invoke {
    pub(crate) fn from_invoke<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        invoke: &'module llvm_ir::terminator::Invoke,
    ) -> Result<Self, Error> {
        Ok(Invoke {
            callee: match &invoke.function {
                Either::Left(_asm) => Callee::Asm,
                Either::Right(op) => Callee::Operand(Operand::new(operands, globals, locals, op)?),
            },
            args: invoke
                .arguments
                .iter()
                .map(|(op, _)| {
                    Operand::new(operands, globals, locals, op).expect("Malformed LLVM module!")
                })
                .collect(),
        })
    }

    pub(crate) fn operands(&self) -> Vec<Arc<Operand>> {
        let mut v = self.args.clone();
        match &self.callee {
            Callee::Operand(o) => v.push(o.clone()),
            Callee::Asm => (),
        };
        v
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Ret {
    pub(crate) operand: Option<Arc<Operand>>,
}

impl Ret {
    pub(crate) fn from_ret<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        ret: &'module llvm_ir::terminator::Ret,
    ) -> Result<Self, Error> {
        Ok(Ret {
            operand: match &ret.return_operand {
                Some(o) => Some(Operand::new(operands, globals, locals, o)?),
                None => None,
            },
        })
    }

    pub(crate) fn operands(&self) -> Vec<Arc<Operand>> {
        match &self.operand {
            Some(o) => vec![o.clone()],
            None => Vec::new(),
        }
    }
}

#[allow(variant_size_differences)] // TODO?
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum TerminatorOpcode {
    Invoke(Invoke),
    Ret(Ret),
    Other,
}

impl TerminatorOpcode {
    pub(crate) fn from_terminator<'module>(
        operands: &mut HashMap<Arc<Operand>, &'module llvm_ir::Operand>,
        globals: &HashMap<&str, Arc<Constant>>,
        locals: &HashMap<&Name, Arc<Operand>>,
        t: &'module llvm_ir::Terminator,
    ) -> Result<Self, Error> {
        Ok(match t {
            llvm_ir::Terminator::Invoke(invoke) => {
                TerminatorOpcode::Invoke(Invoke::from_invoke(operands, globals, locals, invoke)?)
            }
            llvm_ir::Terminator::Ret(ret) => {
                TerminatorOpcode::Ret(Ret::from_ret(operands, globals, locals, ret)?)
            }
            // llvm_ir::Terminator::Br(_) => todo!(),
            // llvm_ir::Terminator::CondBr(_) => todo!(),
            // llvm_ir::Terminator::Switch(_) => todo!(),
            // llvm_ir::Terminator::IndirectBr(_) => todo!(),
            // llvm_ir::Terminator::Resume(_) => todo!(),
            // llvm_ir::Terminator::Unreachable(_) => todo!(),
            // llvm_ir::Terminator::CleanupRet(_) => todo!(),
            // llvm_ir::Terminator::CatchRet(_) => todo!(),
            // llvm_ir::Terminator::CatchSwitch(_) => todo!(),
            // llvm_ir::Terminator::CallBr(_) => todo!(),
            _ => TerminatorOpcode::Other,
        })
    }

    pub(crate) fn operands(&self) -> Vec<Arc<Operand>> {
        match self {
            TerminatorOpcode::Invoke(t) => t.operands(),
            TerminatorOpcode::Ret(t) => t.operands(),
            TerminatorOpcode::Other => Vec::new(),
        }
    }
}
