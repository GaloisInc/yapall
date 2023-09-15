// SPDX-License-Identifier:i BSD-3-Clause
//! Integer analysis

use std::collections::HashMap;

#[cfg(not(feature = "par"))]
use ascent::ascent_run;
#[cfg(feature = "par")]
use ascent::ascent_run_par as ascent_run;

use crate::{
    arc::{Arc, UArc},
    hash::{PreHashed, RefHash},
    klimited::KLimited,
    lattice::{Int, IntLattice},
    llvm::constant::Constant,
    llvm::instruction::{Add, BitCast, Call, IntToPtr, Opcode, Phi, PtrToInt, Select, Sub},
    llvm::{
        BlockName, Callee, FunctionName, InstructionName, InstructionOperand, Invoke, Module,
        Operand, TerminatorOpcode,
    },
};

#[allow(clippy::type_complexity)]
#[derive(Debug)]
pub struct IntRelations {
    pub operand_val: HashMap<(Arc<KLimited<UArc<InstructionName>>>, Arc<Operand>), IntLattice>,
    pub metrics: Option<Metrics>,
}

/// Metrics about the precision of the analysis.
#[derive(Debug)]
pub struct Metrics {
    /// Number of operands marked `Top`
    pub tops: usize,
}

#[allow(clippy::collapsible_if)]
#[allow(clippy::collapsible_match)]
#[allow(clippy::diverging_sub_expression)]
#[allow(clippy::just_underscores_and_digits)]
#[allow(clippy::let_unit_value)]
#[allow(clippy::type_complexity)]
#[allow(clippy::unused_unit)]
pub fn analysis<'module>(
    module: &'module Module,
    callgraph: &HashMap<UArc<InstructionName>, Vec<UArc<FunctionName>>>,
    contexts: usize,
    debug: bool,
    metrics: bool,
) -> IntRelations {
    let main_ctx = Arc::new(KLimited::new(contexts, vec![]));

    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::redundant_clone)]
    #[allow(single_use_lifetimes)]
    #[allow(unreachable_code)]
    #[allow(unused_variables)]
    let outs = ascent_run! {
        struct Outs<'module>;

        // ----------------------------------------------------------
        // AST
        // ----------------------------------------------------------

        // NOTE: See NOTE in andersen.rs re: `Arc<T>`, `PreHashed<_>`,
        // `RefHash<_>`, etc.

        relation function_block_instruction(
            UArc<FunctionName>,
            RefHash<'module, BlockName>,
            InstructionOperand,
            PreHashed<&'module Opcode>,
            llvm_ir::TypeRef);
        function_block_instruction(
            f_name.clone(),
            RefHash::new(&b.name),
            InstructionOperand::new(i.name.clone()),
            i.opcode.for_ref(),
            i.ty.clone(),
        ) <--
          for (f_name, f) in &module.functions,
          for b in &f.blocks,
          for i in &b.instrs;

        relation function_block_terminator(
            UArc<FunctionName>,
            RefHash<'module, BlockName>,
            InstructionOperand,
            PreHashed<&'module TerminatorOpcode>,
            llvm_ir::TypeRef);
        function_block_terminator(
            f_name.clone(),
            RefHash::new(&b.name),
            InstructionOperand::new(b.terminator.name.clone()),
            b.terminator.opcode.for_ref(),
            b.terminator.ty.clone(),
        ) <--
          for (f_name, f) in &module.functions,
          for b in &f.blocks;

        macro function($f: expr) {
           function_block_terminator($f, _, _, _, _)
        }

        macro function_block($f: expr, $b: expr) {
           function_block_terminator($f, $b, _, _, _)
        }

        macro function_instruction($f: expr, $i: expr) {
           function_block_instruction($f, _, $i, _, _)
        }

        macro function_terminator($f: expr, $t: expr) {
           function_block_terminator($f, _, $t, _, _)
        }

        macro block($b: expr) {
           function_block_terminator(_, $b, _, _, _)
        }

        macro instruction($i: expr) {
           function_block_instruction(_, _, $i, _, _)
        }

        macro instruction_opcode($i: expr, $o: expr) {
           function_block_instruction(_, _, $i, $o, _)
        }

        macro terminator_opcode($t: expr, $o: expr) {
           function_block_terminator(_, _, $t, $o, _)
        }

        macro reachable_instruction($ctx: expr, $i: expr) {
           function_block_instruction(f, _, $i, _, _),
           reachable($ctx, f),
        }

        macro reachable_terminator($ctx: expr, $t: expr) {
           function_block_terminator(f, _, $t, _, _),
           reachable($ctx, f),
        }

        macro reachable_instruction_opcode($ctx: expr, $i: expr, $o: expr) {
           function_block_instruction(f, _, $i, $o, _),
           reachable($ctx, f),
        }

        // TODO: Experiment with reversing these
        macro reachable_terminator_opcode($ctx: expr, $t: expr, $o: expr) {
           function_block_terminator(f, _, $t, $o, _),
           reachable($ctx, f),
        }

        // ----------------------------------------------------------
        // Callgraph
        // ----------------------------------------------------------

        relation main(UArc<FunctionName>);

        main(func) <--
          function!(func),
          if **func == "main" ||
             func.starts_with("_ZN") && func.contains("4main"); // rustc

        relation call(InstructionOperand, Arc<Operand>, Arc<Vec<Arc<Operand>>>);

        call(instr, op, Arc::new(args.clone())) <--
          instruction_opcode!(instr, opcode),
          if let Opcode::Call(Call{callee, args}) = &**opcode.as_ref(),
          if let Callee::Operand(op) = callee;

        call(term, op, Arc::new(args.clone())) <--
          terminator_opcode!(term, opcode),
          if let TerminatorOpcode::Invoke(Invoke{callee, args}) = &**opcode.as_ref(),
          if let Callee::Operand(op) = callee;


        // TODO: The destination doesn't depend on the context - split the relation?
        relation calls(
            Arc<KLimited<UArc<InstructionName>>>,
            InstructionOperand,
            UArc<FunctionName>,
            Arc<KLimited<UArc<InstructionName>>>);

        calls(
            ctx,
            instr,
            callee_name.clone(),
            Arc::new(ctx.pushed(instr.instruction_name()))) <--
          reachable_instruction!(ctx, instr),
          if let Some(callees) = callgraph.get(&instr.instruction_name()),
          for callee_name in callees;

        calls(
            ctx,
            instr,
            callee_name.clone(),
            Arc::new(ctx.pushed(instr.instruction_name()))) <--
          reachable_terminator!(ctx, instr),
          if let Some(callees) = callgraph.get(&instr.instruction_name()),
          for callee_name in callees;

        relation reachable(Arc<KLimited<UArc<InstructionName>>>, UArc<FunctionName>);

        reachable(main_ctx.clone(), func) <-- main(func);

        reachable(callee_ctx, callee) <--
          reachable(caller_ctx, caller),
          function_instruction!(caller, instr),
          calls(caller_ctx, instr, callee, callee_ctx);

        // ----------------------------------------------------------
        // Values
        // ----------------------------------------------------------

        lattice operand_val(
            Arc<KLimited<UArc<InstructionName>>>,
            Arc<Operand>,
            IntLattice);

        // ----------------------------------------------------------
        // Constants
        // ----------------------------------------------------------

        operand_val(
            ctx,
            op.clone(),
            IntLattice::constant(Int { value: *value, bits: *bits })) <--
          reachable_instruction_opcode!(ctx, instr, opcode),
          for op in opcode.as_ref().operands(),
          if let Operand::Constant(c) = &*op,
          if let Constant::Int { value, bits } = &**c;

        operand_val(
            ctx,
            op.clone(),
            IntLattice::constant(Int { value: *value, bits: *bits })) <--
          reachable_terminator_opcode!(ctx, instr, opcode),
          for op in opcode.as_ref().operands(),
          if let Operand::Constant(c) = &*op,
          if let Constant::Int { value, bits } = &**c;

        operand_val(ctx, op.clone(), IntLattice::top()) <--
          reachable_instruction_opcode!(ctx, instr, opcode),
          for op in opcode.as_ref().operands(),
          if let Operand::Constant(c) = &*op,
          if !matches!(&**c, Constant::Int { .. });

        operand_val(ctx, op.clone(), IntLattice::top()) <--
          reachable_terminator_opcode!(ctx, instr, opcode),
          for op in opcode.as_ref().operands(),
          if let Operand::Constant(c) = &*op,
          if !matches!(&**c, Constant::Int { .. });

        // ----------------------------------------------------------
        // Pass-thru instructions
        // ----------------------------------------------------------

        relation pass_thru(InstructionOperand, Arc<Operand>);

        pass_thru(i, pointer) <--
          instruction_opcode!(i, opcode),
          if let Opcode::BitCast(BitCast{pointer, ..}) = &**opcode.as_ref();

        pass_thru(i, int) <--
          instruction_opcode!(i, opcode),
          if let Opcode::IntToPtr(IntToPtr{int, ..}) = &**opcode.as_ref();

        pass_thru(i, op) <--
          instruction_opcode!(i, opcode),
          if let Opcode::Phi(Phi{values, ..}) = &**opcode.as_ref(),
          for op in values;

        pass_thru(i, pointer) <--
          instruction_opcode!(i, opcode),
          if let Opcode::PtrToInt(PtrToInt{pointer, ..}) = &**opcode.as_ref();

        pass_thru(i, true_value) <--
          instruction_opcode!(i, opcode),
          if let Opcode::Select(Select{true_value, ..}) = &**opcode.as_ref();

        pass_thru(i, false_value) <--
          instruction_opcode!(i, opcode),
          if let Opcode::Select(Select{false_value, ..}) = &**opcode.as_ref();

        operand_val(ctx, instr.operand(), val) <--
          reachable_instruction!(ctx, instr),
          pass_thru(instr, op),
          operand_val(ctx, op, val);

        // ----------------------------------------------------------
        // Operations
        // ----------------------------------------------------------

        operand_val(ctx, instr.operand(), v0.add(v1)) <--
          reachable_instruction_opcode!(ctx, instr, opcode),
          if let Opcode::Add(Add{operand0, operand1}) = &**opcode.as_ref(),
          operand_val(ctx, operand0, v0),
          operand_val(ctx, operand1, v1);

        operand_val(ctx, instr.operand(), v0.sub(v1)) <--
          reachable_instruction_opcode!(ctx, instr, opcode),
          if let Opcode::Sub(Sub{minuend, subtrahend}) = &**opcode.as_ref(),
          operand_val(ctx, minuend, v0),
          operand_val(ctx, subtrahend, v1);

        operand_val(ctx, instr.operand(), IntLattice::top()) <--
          reachable_instruction_opcode!(ctx, instr, opcode),
          if matches!(&**opcode.as_ref(),
               Opcode::Alloca(_) |
                 Opcode::GetElementPtr(_) |
                 // TODO: more accurate model
                 Opcode::Icmp(_) |
                 // Memory is a black box, who knows what's could be in there!
                 Opcode::Load(_) |
                 Opcode::Other |
                 Opcode::Store(_)
          );

        // ----------------------------------------------------------
        // Unknown
        // ----------------------------------------------------------

        operand_val(ctx, instr.operand(), IntLattice::top()) <--
          reachable_terminator_opcode!(ctx, instr, opcode),
          if let TerminatorOpcode::Other = &**opcode.as_ref();

        // ----------------------------------------------------------
        // Function calls
        // ----------------------------------------------------------

        operand_val(callee_ctx, param, val) <--
          call(call_name, callee_op, args),
          calls(caller_ctx, call_name, callee_name, callee_ctx),
          if let Some(callee) = module.functions.get(callee_name),
          for (i, param) in callee.parameters.iter().enumerate(),
          if let Some(arg) = args.get(i),
          operand_val(caller_ctx, arg, val);

        operand_val(caller_ctx, call_name.operand(), val) <--
          calls(caller_ctx, call_name, callee_name, callee_ctx),
          function_block_terminator(callee_name, _, _, op, _),
          if let TerminatorOpcode::Ret(ret) = &**op.as_ref(),
          if let Some(returned) = &ret.operand,
          operand_val(callee_ctx, returned.clone(), val);

        operand_val(caller_ctx, call_name.operand(), IntLattice::top()) <--
          calls(caller_ctx, call_name, callee_name, callee_ctx),
          if let Some(callee) = module.functions.get(callee_name),
          if let llvm_ir::Type::VoidType = &*callee.return_type;

        operand_val(caller_ctx, call_name.operand(), IntLattice::top()) <--
          calls(caller_ctx, call_name, callee_name, callee_ctx),
          if let Some(callee) = module.decls.get(callee_name),
          if let llvm_ir::Type::VoidType = &*callee.return_type;

        // Calls to external functions have unknown return values
        operand_val(caller_ctx, call_name.operand(), IntLattice::top()) <--
          calls(caller_ctx, call_name, callee_name, callee_ctx),
          if module.decls.get(callee_name).is_some();

        // ----------------------------------------------------------
        // argc
        // ----------------------------------------------------------

        operand_val(main_ctx.clone(), argc, IntLattice::top()) <--
          main(main_name),
          if let Some(func) = module.functions.get(main_name),
          if let Some(argc) = func.parameters.get(0);

        // ----------------------------------------------------------
        // Assertions
        // ----------------------------------------------------------

        relation _assert_call();
        _assert_call() <--
          reachable_instruction_opcode!(ctx, i, opcode),
          if let Opcode::Call { .. } = opcode.as_ref(),
          !calls(_, i, _, _),
          let _ = panic!("Bug! Call without target: {}", i.instruction_name());

        relation _assert_invoke();
        _assert_invoke() <--
          reachable_terminator_opcode!(ctx, i, opcode),
          if let TerminatorOpcode::Invoke { .. } = opcode.as_ref(),
          !calls(_, i, _, _),
          let _ = panic!("Bug! Invoke without target: {}", i.instruction_name());

        relation _assert_reachable_vals_have_values();
        _assert_reachable_vals_have_values() <--
          function_block_instruction(f, _b, i, _o, t),
          reachable(ctx, f),
          !operand_val(ctx, i.operand(), _),
          let _ = panic!("Bug! Instruction doesn't have a value: {}", i.instruction_name());

        // ----------------------------------------------------------
        // Metrics
        // ----------------------------------------------------------

        relation tops(Arc<Operand>);
        tops(op) <-- operand_val(_, op, IntLattice::top());
    };

    if debug {
        eprintln!("{}", outs.summary());
        eprintln!("{}", outs.scc_times_summary());
    }

    #[cfg(not(feature = "par"))]
    fn unwrap_lock<T>(x: T) -> T {
        x
    }

    #[cfg(feature = "par")]
    fn unwrap_lock<T>(x: std::sync::RwLock<T>) -> T {
        x.into_inner().unwrap()
    }

    IntRelations {
        operand_val: outs
            .operand_val
            .into_iter()
            .map(unwrap_lock)
            .map(|tup| ((tup.0, tup.1), tup.2))
            // .filter(|((_, op), _)| !matches!(&**op, Operand::Constant(_)))
            .collect(),
        metrics: if metrics {
            Some(Metrics {
                tops: outs.tops.len(),
            })
        } else {
            None
        },
    }
}
