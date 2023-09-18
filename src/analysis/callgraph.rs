// SPDX-License-Identifier: BSD-3-Clause
//! Callgraph analysis
//!
//! TODO:
//!
//! - Handle global aliases?
//! - Fix unsoundness around certain constants (marked with TODO)
//! - Perhaps incorporate function argument types?
//! - Only add indirect call targets for functions that have their address taken?

use std::collections::HashMap;

use crate::{
    arc::UArc,
    llvm::constant::Constant,
    llvm::instruction::Opcode,
    llvm::{Callee, FunctionName, InstructionName, Module, Operand, TerminatorOpcode},
};

fn constant_functions(c: &Constant) -> Vec<UArc<FunctionName>> {
    match c {
        Constant::Function(f) => vec![f.clone()],
        Constant::BitCast(b) => constant_functions(&b.pointer),
        c @ Constant::GetElementPtr(_) => {
            c.pointers().iter().flat_map(constant_functions).collect()
        }
        // No `_` pattern to ensure this is updated if the type changes
        Constant::Global(_) => Vec::new(),
        Constant::Int { .. } => Vec::new(),
        Constant::Null => Vec::new(),
        Constant::Add => Vec::new(),
        Constant::Sub => Vec::new(),
        Constant::Mul => Vec::new(),
        Constant::UDiv => Vec::new(),
        Constant::SDiv => Vec::new(),
        Constant::URem => Vec::new(),
        Constant::SRem => Vec::new(),
        Constant::And => Vec::new(),
        Constant::Or => Vec::new(),
        Constant::Xor => Vec::new(),
        Constant::Shl => Vec::new(),
        Constant::LShr => Vec::new(),
        Constant::AShr => Vec::new(),
        Constant::FAdd => Vec::new(),
        Constant::FSub => Vec::new(),
        Constant::FMul => Vec::new(),
        Constant::FDiv => Vec::new(),
        Constant::FRem => Vec::new(),
        Constant::ExtractElement => Vec::new(), // TODO
        Constant::InsertElement => Vec::new(),
        Constant::ShuffleVector => Vec::new(),
        Constant::ExtractValue => Vec::new(), // TODO
        Constant::InsertValue => Vec::new(),
        Constant::Trunc => Vec::new(),
        Constant::ZExt => Vec::new(),
        Constant::SExt => Vec::new(),
        Constant::FPTrunc => Vec::new(),
        Constant::FPExt => Vec::new(),
        Constant::FPToUI => Vec::new(),
        Constant::FPToSI => Vec::new(),
        Constant::UIToFP => Vec::new(),
        Constant::SIToFP => Vec::new(),
        Constant::PtrToInt(_) => Vec::new(),   // TODO
        Constant::IntToPtr(_) => Vec::new(),   // TODO
        Constant::AddrSpaceCast => Vec::new(), // TODO
        Constant::ICmp => Vec::new(),
        Constant::FCmp => Vec::new(),
        Constant::Select => Vec::new(), // TODO
        Constant::Array(_) => Vec::new(),
        Constant::Struct(_) => Vec::new(),
        Constant::Undef => Vec::new(), // TODO
        Constant::Other => Vec::new(), // TODO
    }
}

pub(crate) fn indirect_call_targets(
    module: &Module,
    nargs: usize,
) -> impl Iterator<Item = UArc<FunctionName>> + '_ {
    module
        .functions
        .iter()
        // TODO: filter_map
        .filter(move |(_f_name, f)| nargs >= f.parameters.len())
        .map(|(f_name, _)| f_name.clone())
        .chain(
            module
                .decls
                .iter()
                // TODO: filter_map
                .filter(move |(_f_name, f)| nargs >= f.parameters.len())
                .map(|(f_name, _)| f_name.clone()),
        )
}

/// Get all possible targets of a call.
///
/// Rules:
///
/// - Direct calls have the obvious target, calls through other constants are
///   explained above.
/// - Calls to assembly are treated as possibly calling any function.
/// - Indirect calls through variables may target any function with fewer
///   parameters (one might think it would be enough to consider functions
///   with *exactly* the same number of parameters, but in practice, calling a
///   function with extra parameters is generally OK).
pub(crate) fn call_targets(
    module: &Module,
    callee: &Callee,
    nargs: usize,
) -> Vec<UArc<FunctionName>> {
    match callee {
        Callee::Asm => indirect_call_targets(module, usize::MAX).collect(),
        Callee::Operand(o) => match &**o {
            Operand::Metadata => {
                debug_assert!(false);
                Vec::new()
            }
            Operand::Constant(c) => {
                let mut fs = constant_functions(c);
                assert!(!fs.is_empty());
                fs.shrink_to_fit();
                fs
            }
            Operand::Local(_) => indirect_call_targets(module, nargs).collect(),
        },
    }
}

/// Over-approximate callgraph analysis
pub fn analysis(module: &Module) -> HashMap<UArc<InstructionName>, Vec<UArc<FunctionName>>> {
    // Size heuristic: Most functions will be called at least once. This is
    // pretty conservative, as many functions will be called many times.
    let mut m = HashMap::with_capacity(module.functions.len());
    for f in module.functions.values() {
        for b in &f.blocks {
            match b.terminator.opcode.as_ref() {
                TerminatorOpcode::Invoke(i) => {
                    let targets = call_targets(module, &i.callee, i.args.len());
                    assert!(!targets.is_empty());
                    m.insert(b.terminator.name.clone(), targets);
                }
                // No `_` pattern to ensure this is updated if the type changes
                TerminatorOpcode::Ret(_) => (),
                TerminatorOpcode::Other => (),
            };
            for i in &b.instrs {
                match i.opcode.as_ref() {
                    Opcode::Call(c) => {
                        let targets = call_targets(module, &c.callee, c.args.len());
                        assert!(!targets.is_empty());
                        m.insert(i.name.clone(), targets);
                    }
                    // No `_` pattern to ensure this is updated if the type changes
                    Opcode::Add(_) => (),
                    Opcode::Alloca(_) => (),
                    Opcode::BitCast(_) => (),
                    Opcode::GetElementPtr(_) => (),
                    Opcode::Icmp(_) => (),
                    Opcode::IntToPtr(_) => (),
                    Opcode::Load(_) => (),
                    Opcode::Phi(_) => (),
                    Opcode::PtrToInt(_) => (),
                    Opcode::Select(_) => (),
                    Opcode::Store(_) => (),
                    Opcode::Sub(_) => (),
                    Opcode::Other => (),
                };
            }
        }
    }
    m
}
