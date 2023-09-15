// SPDX-License-Identifier:i BSD-3-Clause
// TODO: Handle global aliases
use std::collections::{HashMap, HashSet};

#[cfg(not(feature = "par"))]
use ascent::ascent_run;
#[cfg(feature = "par")]
use ascent::ascent_run_par as ascent_run;

use tracing::trace_span;

use crate::{
    alloc::{Alloc, FunctionAlloc, GlobalAlloc, HeapAlloc, StackAlloc},
    analysis::callgraph::indirect_call_targets,
    arc::{Arc, UArc},
    hash::PreHashed,
    klimited::KLimited,
    llvm::constant::Constant,
    llvm::instruction::{
        Add, BitCast, Call, GetElementPtr, IntToPtr, Load, Opcode, Phi, PtrToInt, Select, Store,
        Sub,
    },
    llvm::{
        Callee, FunctionName, GlobalName, InstructionName, InstructionOperand, Invoke, Module,
        Operand, TerminatorOpcode,
    },
    signatures::{AllocType, Signature, Signatures},
};

#[allow(clippy::type_complexity)]
#[derive(Debug)]
pub struct OutputRelations {
    pub alloc_points_to: Vec<(Arc<Alloc>, Arc<Alloc>)>,
    pub operand_points_to: Vec<(
        Arc<KLimited<UArc<InstructionName>>>,
        Arc<Operand>,
        Arc<Alloc>,
    )>,
    pub reachable: Vec<UArc<FunctionName>>,
    pub calls:
        HashMap<(Arc<KLimited<UArc<InstructionName>>>, UArc<InstructionName>), UArc<FunctionName>>,
    pub needs_signature: Vec<UArc<FunctionName>>,
    pub metrics: Option<Metrics>,
}

// TODO: Metrics for memset of too-small allocations
//
/// Metrics about the precision of the points-to analysis. Lower is better.
#[derive(Debug)]
pub struct Metrics {
    /// Number of callgraph edges, that is, edges from a call-like instruction
    /// (`call`, `invoke`, etc.) to possible callee functions. A more precise
    /// analysis will resolve indirect calls more accurately, leading to
    /// a smaller callgraph.
    pub callgraph_size: usize,
    /// Non-heap (stack, global, function) allocations passed to `free`. This
    /// is undefined behavior, so it must reflect an imprecision in the
    /// analysis.
    pub free_non_heap: usize,
    /// Calls to non-function pointers. As this can never happen in a
    /// well-defined program (it would cause a segfault or invalid read,
    /// respectively), it must reflect an imprecision in the analysis.
    pub invalid_calls: usize,
    /// Operands that are nullable or point to functions, but are loaded from.
    /// As this can never happen in a well-defined program (it would cause a
    /// segfault or invalid read, respectively), it must reflect an imprecision
    /// in the analysis.
    pub invalid_loads: usize,
    /// `memcpy` calls where the destination allocation was not storable (e.g.,
    /// was null or const) or was not big enough. As this can never happen in
    /// a well-defined program (it would cause a segfault), it must reflect an
    /// imprecision in the analysis.
    pub invalid_memcpy_dsts: usize,
    /// `memcpy` calls where the source allocation was not loadable (e.g., was
    /// null) or was not big enough. As this can never happen in a well-defined
    /// program (it would cause a segfault), it must reflect an imprecision in
    /// the analysis.
    pub invalid_memcpy_srcs: usize,
    /// Operands that are nullable, point to constant global allocations, or
    /// point to functions, but are stored to. As this can never happen in a
    /// well-defined program (it would cause a segfault), it must reflect an
    /// imprecision in the analysis.
    pub invalid_stores: usize,
    /// Operands that point to the special `Top` allocation. `Top` is used
    /// to model complex language features; more precise models should be
    /// preferred.
    pub points_to_top: usize,
}

#[derive(Debug)]
pub struct Options {
    pub check_assertions: bool,
    pub check_strict: bool,
    pub contexts: u8,
    pub debug: bool,
    pub metrics: bool,
    pub unification: bool,
}

// Profiling machinery
#[inline]
#[allow(unused_variables)]
fn count(relation: &str, rule: &str) -> bool {
    #[cfg(all(feature = "count", feature = "relation"))]
    eprintln!("{} 1", relation);
    #[cfg(all(feature = "count", feature = "rule"))]
    eprintln!("{} 1", rule);
    true
}

/// Pointer analysis
///
/// Sources of unsoundness:
///
/// - External functions without signatures
/// - Signatures not expressive enough to model external functions (e.g.,
///   `getline`, functions that take a callback)
/// - C++ exceptions
/// - Variable-arity functions
#[allow(clippy::collapsible_if)]
#[allow(clippy::collapsible_match)]
#[allow(clippy::diverging_sub_expression)]
#[allow(clippy::just_underscores_and_digits)]
#[allow(clippy::let_unit_value)]
#[allow(clippy::type_complexity)]
#[allow(clippy::unused_unit)]
pub fn analysis<'module>(
    module: &'module Module,
    signatures: &Signatures,
    opts: &Options,
) -> OutputRelations {
    {
        #![allow(clippy::nonminimal_bool)]
        debug_assert!(!(opts.check_strict && !opts.check_assertions));
    }

    let argv_alloc = Arc::new(Alloc::Global(GlobalAlloc::new(
        Arc::new(GlobalName::from("*argv")),
        false,
        None,
    )));
    let argv0_alloc = Arc::new(Alloc::Global(GlobalAlloc::new(
        Arc::new(GlobalName::from("*argv[*]")),
        false,
        None,
    )));
    let optarg_alloc = Arc::new(Alloc::Global(GlobalAlloc::new(
        Arc::new(GlobalName::from("*@optarg")),
        false,
        None,
    )));
    let stderr_alloc = Arc::new(Alloc::Global(GlobalAlloc::new(
        Arc::new(GlobalName::from("*@stderr")),
        false,
        None,
    )));
    let stdin_alloc = Arc::new(Alloc::Global(GlobalAlloc::new(
        Arc::new(GlobalName::from("*@stdin")),
        false,
        None,
    )));
    let stdout_alloc = Arc::new(Alloc::Global(GlobalAlloc::new(
        Arc::new(GlobalName::from("*@stdout")),
        false,
        None,
    )));

    // This one is used by a signature:
    let ctype_b_loc_alloc = Arc::new(Alloc::Global(GlobalAlloc::new(
        Arc::new(GlobalName::from("__ctype_b_loc_alloc")),
        false,
        None,
    )));
    let ctype_b_loc_alloc_alloc = Arc::new(Alloc::Global(GlobalAlloc::new(
        Arc::new(GlobalName::from("__ctype_b_loc_alloc_alloc")),
        false,
        None,
    )));

    let null_alloc = Arc::new(Alloc::Null);
    let top = Arc::new(Alloc::Top);

    let main_ctx = Arc::new(KLimited::new(opts.contexts.into(), vec![]));

    // Preprocess the signatures, specializing to the ones for declarations in
    // this module.
    // TODO: Use a ref to functionname?
    let mut sigs: HashMap<UArc<FunctionName>, Vec<Signature>> = HashMap::new();
    for f in module.decls.keys() {
        if let Some(s) = signatures.signatures_for((*f).get()) {
            sigs.insert(f.clone(), s);
        }
    }

    let known_functions = HashSet::from([
        // Used in tests:
        FunctionName::from("assert_disjoint"),
        FunctionName::from("assert_may_alias"),
        FunctionName::from("assert_points_to_nothing"),
        FunctionName::from("assert_points_to_something"),
        FunctionName::from("assert_reachable"),
        FunctionName::from("assert_unreachable"),
        // Standard:
        FunctionName::from("__memcpy_chk"),
        FunctionName::from("calloc"),
        FunctionName::from("free"),
        FunctionName::from("realloc"),
        FunctionName::from("reallocarray"),
        FunctionName::from("malloc"),
        FunctionName::from("_Znwm"),
        // TODO: Signatures for modeling this kind of behavior
        FunctionName::from("strtol"),
        FunctionName::from("strtoll"),
        FunctionName::from("strtoul"),
        // TODO: Models for these functions
        // @__cxa_throw
        // @__cxa_free_exception
        // @__cxa_begin_catch
        // @__cxa_allocate_exception
        // @__cxa_end_catch
        // @_ZSt9terminatev
        // @llvm.eh.typeid.for
        // @_ZNSt11logic_errorC1EPKc
        // @__gxx_personality_v0
        // @_ZNSt11logic_errorD1Ev
    ]);

    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::redundant_clone)]
    #[allow(single_use_lifetimes)]
    #[allow(unreachable_code)]
    #[allow(unused_variables)]
    let outs = ascent_run! {
        #![measure_rule_times]

        struct Outs<'module>;

        // TODO: undef and poison

        // ----------------------------------------------------------
        // AST
        // ----------------------------------------------------------

        // NOTE: The analysis uses several "wrapper" types.
        //
        // - `Arc<T>` is like `std::sync::Arc<T>` in that it provides thread-
        //   safe shared ownership. However, it also provides a precomputed
        //   hash, as Ascent needs to hash values all the time and many parts
        //   of the LLVM module structure are quite large.
        // - `PreHashed<&T>` is like `Arc<T>` without the ownership. It should
        //   be generally more efficient than `Arc<T>`, as it avoids a clone
        //   on creation and the (atomic) reference counting logic. However,
        //   it can only be used for values that aren't produced during the
        //   analysis (i.e., those that can be borrowed instead) (e.g., not
        //   `Alloc` nor `GlobalName`).
        // - `RefHash<T>` stores a `&T`, and uses pointer equality and
        //   comparison. For values that can be uniquely identified by
        //   their location in memory, this should be very fast and memory
        //   efficient.
        // - `UArc<T>` is like `std::sync::Arc<T>`, but uses pointer equality,
        //   comparison, and hashing. For values that are created in only one
        //   spot, this should be very fast and memory efficient.

        relation function_instruction_opcode(
            UArc<FunctionName>,
            InstructionOperand,
            PreHashed<&'module Opcode>,
            llvm_ir::TypeRef);
        function_instruction_opcode(
            f_name.clone(),
            InstructionOperand::new(i.name.clone()),
            i.opcode.for_ref(),
            i.ty.clone(),
        ) <--
          for (f_name, f) in &module.functions,
          for b in &f.blocks,
          for i in &b.instrs;

        relation function_terminator_opcode(
            UArc<FunctionName>,
            InstructionOperand,
            PreHashed<&'module TerminatorOpcode>,
            llvm_ir::TypeRef);
        function_terminator_opcode(
            f_name.clone(),
            InstructionOperand::new(b.terminator.name.clone()),
            b.terminator.opcode.for_ref(),
            b.terminator.ty.clone(),
        ) <--
          for (f_name, f) in &module.functions,
          for b in &f.blocks;

        macro function($f: expr) {
           function_terminator_opcode($f, _, _, _)
        }

        macro function_instruction($f: expr, $i: expr) {
           function_instruction_opcode($f, $i, _, _)
        }

        macro function_terminator($f: expr, $t: expr) {
           function_terminator_opcode($f, $t, _, _)
        }

        macro instruction($i: expr) {
           function_instruction_opcode(_, $i, _, _)
        }

        macro instruction_opcode($i: expr, $o: expr) {
           function_instruction_opcode(_, $i, $o, _)
        }

        macro terminator_opcode($t: expr, $o: expr) {
           function_terminator_opcode(_, $t, $o, _)
        }

        // TODO: Experiment with reversing these
        macro reachable_instruction($ctx: expr, $i: expr) {
           function_instruction_opcode(f, $i, _, _),
           reachable($ctx, f),
        }

        // TODO: Experiment with reversing these
        macro reachable_instruction_opcode($ctx: expr, $i: expr, $o: expr) {
           function_instruction_opcode(f, $i, $o, _),
           reachable($ctx, f),
        }

        // TODO: Experiment with reversing these
        macro reachable_terminator_opcode($ctx: expr, $t: expr, $o: expr) {
           function_terminator_opcode(f, $t, $o, _),
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

        call(instr, op.clone(), Arc::new(args.clone())) <--
          instruction_opcode!(instr, opcode),
          if let Opcode::Call(Call{callee, args}) = &**opcode.as_ref(),
          if let Callee::Operand(op) = callee;

        call(term, op.clone(), Arc::new(args.clone())) <--
          terminator_opcode!(term, opcode),
          if let TerminatorOpcode::Invoke(Invoke{callee, args}) = &**opcode.as_ref(),
          if let Callee::Operand(op) = callee;

        relation calls(
            Arc<KLimited<UArc<InstructionName>>>,
            InstructionOperand,
            UArc<FunctionName>,
            Arc<Vec<Arc<Operand>>>,
            Arc<KLimited<UArc<InstructionName>>>);

        calls(
            ctx,
            instr,
            func_alloc.function_name(),
            args,
            Arc::new(ctx.pushed(instr.instruction_name()))) <--
          let span = trace_span!("calls"),
          let _span = span.enter(),
          //
          call(instr, callee_op, args),
          operand_points_to(ctx, callee_op, alloc),
          if let Alloc::Function(func_alloc) = &**alloc,
          //
          if count("calls", "calls");

        // Conservative handling of calls through `Top`
        calls(
            ctx,
            instr,
            f.clone(),
            args,
            Arc::new(ctx.pushed(instr.instruction_name()))) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "calls"
          } else {
              "top"
          }),
          let _span = span.enter(),
          //
          //
          call(instr, callee_op, args),
          // TODO: Does this capture `top` if I just use it as a variable?
          operand_points_to(ctx, callee_op, top.clone()),
          for f in indirect_call_targets(module, args.len()),
          //
          if count("calls", "top");

        relation reachable(Arc<KLimited<UArc<InstructionName>>>, UArc<FunctionName>);

        reachable(main_ctx.clone(), func) <-- main(func);

        reachable(callee_ctx, callee) <--
          let span = trace_span!("reachable"),
          let _span = span.enter(),
          //
          reachable(caller_ctx, caller),
          function_instruction!(caller, instr),
          calls(caller_ctx, instr, callee, _, callee_ctx),
          //
          if count("reachable", "reachable");

        // ----------------------------------------------------------
        // Allocations
        // ----------------------------------------------------------

        relation global_alloc(Arc<GlobalName>, Arc<GlobalAlloc>);

        global_alloc(
            g_name,
            GlobalAlloc::new(g_name.clone(), g.is_const, g.size())) <--
          for (g_name, g) in &module.globals;

        // heap
        operand_points_to(
            ctx,
            i.operand(),
            Arc::new(Alloc::Heap(HeapAlloc::new(i.instruction_name(), sz)))) <--
          calls(ctx, i, f, args, _),
          for (name, size_arg) in [
              ("_Znwm", Some(0)),
              ("calloc", None),
              ("malloc", Some(0)),
              ("realloc", Some(1)),
              ("reallocarray", None),
          ],
          if **f == name,
          let sz = size_arg.and_then(
              |i| args.get(i).and_then(|op| op.constant_int())
          );

        // stack
        operand_points_to(
            ctx,
            i.operand(),
            Arc::new(Alloc::Stack(StackAlloc::alloca(i.instruction_name(), a)))) <--
          reachable_instruction_opcode!(ctx, i, opcode),
          if let Opcode::Alloca(a) = &&**opcode.as_ref();

        // ----------------------------------------------------------
        // Points-to
        // ----------------------------------------------------------

        relation operand_points_to(
            Arc<KLimited<UArc<InstructionName>>>,
            Arc<Operand>,
            Arc<Alloc>);

        operand_points_to(
            ctx,
            op.clone(),
            Alloc::lookup(a)) <--
          reachable_instruction_opcode!(ctx, instr, opcode),
          for op in &opcode.as_ref().operands(),
          if let Operand::Constant(c) = &**op,
          constant_points_to(c, a);

        operand_points_to(
            ctx,
            op.clone(),
            Alloc::lookup(a)) <--
          reachable_terminator_opcode!(ctx, instr, opcode),
          for op in opcode.as_ref().operands(),
          if let Operand::Constant(c) = &*op,
          constant_points_to(c, a);

        relation constant_points_to(Arc<Constant>, Arc<Alloc>);

        constant_points_to(
            Arc::new(Constant::Function(f.clone())),
            Arc::new(Alloc::Function(FunctionAlloc::new(f.clone())))) <--
          for f in module.functions.keys();

        constant_points_to(
            Arc::new(Constant::Function(f.clone())),
            Arc::new(Alloc::Function(FunctionAlloc::new(f.clone())))) <--
          for f in module.decls.keys();

        constant_points_to(
            Arc::new(Constant::Global(g.clone())),
            Arc::new(Alloc::Global(a.clone()))) <--
          global_alloc(g, a);

        // Constant operations (e.g., bitcast, ptrtoint, getelementptr) pass
        // through the points-to facts from their operands.

        constant_points_to(c0.clone(), Alloc::lookup(a)) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "constant_points_to"
          } else {
              "constant"
          }),
          let _span = span.enter(),
          //
          instruction_opcode!(instr, opcode),
          for op in opcode.as_ref().operands(),
          if let Operand::Constant(c0) = &*op,
          for c in c0.pointers(),
          constant_points_to(Arc::new(c), a),
          //
          if count("constant_points_to", "constant");

        constant_points_to(init.clone(), Alloc::lookup(a)) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "constant_points_to"
          } else {
              "constant_init"
          }),
          let _span = span.enter(),
          //
          for g in module.globals.values(),
          if let Some(init) = &g.initializer,
          for c in init.pointers(),
          constant_points_to(Arc::new(c), a),
          //
          if count("constant_points_to", "constant_init");

        // ----------------------------------------------------------
        // Merging
        // ----------------------------------------------------------

        relation merge(Arc<Alloc>);

        merge(c) <--
          if opts.unification,
          operand_points_to(ctx, i, a),
          operand_points_to(ctx, i, b),
          if a.merge(b),
          for c in [a, b];

        alloc_points_to(Alloc::lookup(a), b) <--
          if opts.unification,
          merge(a),
          alloc_points_to(a, b);

        alloc_points_to(a, b) <--
          if opts.unification,
          merge(a),
          alloc_points_to(Alloc::lookup(a), b);

        // ----------------------------------------------------------
        // Special allocations
        // ----------------------------------------------------------

        // Null is modeled as pointing to a special "null allocation". This
        // makes it easy to track nullability, e.g., across function calls,
        // loads, etc., without duplicating a bunch of rules.
        //
        // This strategy does require more care when interpreting the results
        // of the points-to analysis. For example, two nullable pointers
        // with otherwise disjoint points-to sets shouldn't be considered as
        // possibly aliasing.
        //
        // The null allocation doesn't point to anything, because loading
        // from or storing to it would be undefined behavior, which we assume
        // doesn't happen in the program under analysis.
        constant_points_to(Arc::new(Constant::Null), null_alloc) <-- if true;

        // Loading from `Top` also yields `Top`.
        alloc_points_to(top.clone(), top.clone()) <-- if true;

        // ----------------------------------------------------------
        // Stores and loads
        // ----------------------------------------------------------

        relation load(InstructionOperand, Arc<Operand>);

        load(instr, pointer.clone()) <--
          instruction_opcode!(instr, opcode),
          if let Opcode::Load(Load{pointer, ..}) = &**opcode.as_ref();

        relation store(InstructionOperand, Arc<Operand>, Arc<Operand>);

        store(instr, pointer.clone(), value.clone()) <--
          instruction_opcode!(instr, opcode),
          if let Opcode::Store(Store{pointer, value, ..}) = &**opcode.as_ref();

        relation alloc_points_to(Arc<Alloc>, Arc<Alloc>);

        // Store instructions
        alloc_points_to(pointer_alloc, Alloc::lookup(pointee_alloc)) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "alloc_points_to"
          } else {
              "store"
          }),
          let _span = span.enter(),
          //
          store(instr, pointer, value),
          operand_points_to(ctx, value, pointee_alloc),
          operand_points_to(ctx, pointer, pointer_alloc),
          if pointer_alloc.storable(),
          //
          if count("alloc_points_to", "store");

        // Load instructions
        operand_points_to(ctx, instr.operand(), Alloc::lookup(pointee_alloc)) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "operand_points_to"
          } else {
              "load"
          }),
          let _span = span.enter(),
          //
          load(instr, pointer),
          operand_points_to(ctx, pointer, pointer_alloc),
          alloc_points_to(pointer_alloc, pointee_alloc),
          //
          if count("operand_points_to", "load");

        // ----------------------------------------------------------
        // Globals
        // ----------------------------------------------------------

        alloc_points_to(Arc::new(Alloc::Global(g_alloc.clone())), Alloc::lookup(a)) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "alloc_points_to"
          } else {
              "global_init"
          }),
          let _span = span.enter(),
          //
          for (g_name, g) in &module.globals,
          if let Some(init) = &g.initializer,
          let init_operand = Arc::new(Operand::Constant(init.clone())),
          global_alloc(g_name, g_alloc),
          constant_points_to(init, a),
          //
          if count("alloc_points_to", "global_init");

        // ----------------------------------------------------------
        // Pass-thru instructions
        // ----------------------------------------------------------

        relation pass_thru(InstructionOperand, Arc<Operand>);

        pass_thru(i, operand0) <--
          instruction_opcode!(i, opcode),
          if let Opcode::Add(Add{operand0, ..}) = &**opcode.as_ref();

        pass_thru(i, operand1) <--
          instruction_opcode!(i, opcode),
          if let Opcode::Add(Add{operand1, ..}) = &**opcode.as_ref();

        pass_thru(i, pointer) <--
          instruction_opcode!(i, opcode),
          if let Opcode::BitCast(BitCast{pointer, ..}) = &**opcode.as_ref(),
          // See NOTE[pass-thru]
          if !matches!(pointer.as_ref(), Operand::Local(_));

        pass_thru(i, pointer) <--
          instruction_opcode!(i, opcode),
          if let Opcode::GetElementPtr(GetElementPtr{pointer, ..}) = &**opcode.as_ref(),
          // See NOTE[pass-thru]
          if !matches!(pointer.as_ref(), Operand::Local(_));

        pass_thru(i, int) <--
          instruction_opcode!(i, opcode),
          // See NOTE[pass-thru]
          if let Opcode::IntToPtr(IntToPtr{int, ..}) = &**opcode.as_ref(),
          if !matches!(int.as_ref(), Operand::Local(_));

        pass_thru(i, op) <--
          instruction_opcode!(i, opcode),
          if let Opcode::Phi(Phi{values, ..}) = &**opcode.as_ref(),
          for op in values;

        pass_thru(i, pointer) <--
          instruction_opcode!(i, opcode),
          // See NOTE[pass-thru]
          if let Opcode::PtrToInt(PtrToInt{pointer, ..}) = &**opcode.as_ref(),
          if !matches!(pointer.as_ref(), Operand::Local(_));

        pass_thru(i, true_value) <--
          instruction_opcode!(i, opcode),
          if let Opcode::Select(Select{true_value, ..}) = &**opcode.as_ref();

        pass_thru(i, false_value) <--
          instruction_opcode!(i, opcode),
          if let Opcode::Select(Select{false_value, ..}) = &**opcode.as_ref();

        pass_thru(i, minuend) <--
          instruction_opcode!(i, opcode),
          if let Opcode::Sub(Sub{minuend, ..}) = &**opcode.as_ref();

        operand_points_to(ctx, i.operand(), Alloc::lookup(a)) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "operand_points_to"
          } else {
              "pass_thru"
          }),
          let _span = span.enter(),
          //
          pass_thru(i, op),
          operand_points_to(ctx, op, a),
          //
          if count("operand_points_to", "pass_thru");

        // ----------------------------------------------------------
        // Function calls
        // ----------------------------------------------------------

        operand_points_to(callee_ctx, param, Alloc::lookup(a)) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "operand_points_to"
          } else {
              "call_arg"
          }),
          let _span = span.enter(),
          //
          calls(caller_ctx, call_name, callee_name, args, callee_ctx),
          if let Some(callee) = module.functions.get(callee_name),
          for (i, param) in callee.parameters.iter().enumerate(),
          if let Some(arg) = args.get(i),
          operand_points_to(caller_ctx, arg, a),
          //
          if count("operand_points_to", "call_arg");

        // The below operand_points_to rule for returns is actually a hot
        // spot for the analysis. Therefore, we split up the work with this
        // relation.
        relation returns(UArc<FunctionName>, Arc<Operand>);

        returns(func, returned) <--
          function_terminator_opcode(func, _, op, _),
          if let TerminatorOpcode::Ret(ret) = &**op.as_ref(),
          if let Some(returned) = &ret.operand;

        operand_points_to(caller_ctx, call_name.operand(), Alloc::lookup(a)) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "operand_points_to"
          } else {
              "call_ret"
          }),
          let _span = span.enter(),
          //
          calls(caller_ctx, call_name, callee, _, callee_ctx),
          returns(callee, returned),
          operand_points_to(callee_ctx, returned, a),
          //
          if count("operand_points_to", "call_ret");

        // ----------------------------------------------------------
        // memcpy
        // ----------------------------------------------------------

        relation memcpy(
            Arc<KLimited<UArc<InstructionName>>>,
            Arc<Operand>,
            Arc<Operand>,
            Option<u64>);

        memcpy(ctx, dst, src, sz) <--
          let span = trace_span!("memcpy"),
          let _span = span.enter(),
          calls(ctx, i, f, args, _),
          if **f == "memcpy" ||
             **f == "__memcpy_chk" ||
             f.starts_with("llvm.memcpy") ||
             f.starts_with("llvm.memmove"),
          if let Some(dst) = args.get(0),
          if let Some(src) = args.get(1),
          let sz = {
              if let Some(op_arc) = args.get(2) {
                (**op_arc).constant_int()
              } else {
                  None
              }
          },
          //
          if count("memcpy", "memcpy");

        relation memcpy_alloc(Arc<Alloc>, Arc<Alloc>);
        memcpy_alloc(Alloc::lookup(dst_alloc), Alloc::lookup(src_alloc)) <--
          let span = trace_span!("memcpy_alloc"),
          let _span = span.enter(),
          //
          memcpy(ctx, dst, src, sz),
          let min_size = sz.unwrap_or(0),
          operand_points_to(ctx, src, src_alloc),
          if src_alloc.loadable(),
          operand_points_to(ctx, dst, dst_alloc),
          if dst_alloc.storable(),
          //
          if count("memcpy_alloc", "memcpy_alloc");

        alloc_points_to(dst_alloc, a) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "alloc_points_to"
          } else {
              "alloc_memcpy"
          }),
          let _span = span.enter(),
          //
          memcpy_alloc(dst_alloc, src_alloc),
          alloc_points_to(src_alloc, a),
          //
          if count("alloc_points_to", "alloc_memcpy");

        // ----------------------------------------------------------
        // argv
        // ----------------------------------------------------------

        operand_points_to(main_ctx.clone(), argv, argv_alloc.clone()) <--
          main(main_name),
          if let Some(func) = module.functions.get(main_name),
          if let Some(argv) = func.parameters.get(1);

        alloc_points_to(argv_alloc, argv0_alloc) <--
          if true;

        // ----------------------------------------------------------
        // stderr, stdin, stdout, __ctype_b_loc
        // ----------------------------------------------------------

        // TODO: Some kind of signature for these

        alloc_points_to(
            Arc::new(Alloc::Global(g_alloc.clone())),
            optarg_alloc.clone()) <--
          for (g_name, g) in &module.globals,
          if **g_name == GlobalName::from("optarg"),
          if g.initializer.is_none(),
          global_alloc(g_name, g_alloc);

        alloc_points_to(
            Arc::new(Alloc::Global(g_alloc.clone())),
            stderr_alloc.clone()) <--
          for (g_name, g) in &module.globals,
          if **g_name == GlobalName::from("stderr"),
          if g.initializer.is_none(),
          global_alloc(g_name, g_alloc);

        alloc_points_to(
            Arc::new(Alloc::Global(g_alloc.clone())),
            stdin_alloc.clone()) <--
          for (g_name, g) in &module.globals,
          if **g_name == GlobalName::from("stdin"),
          if g.initializer.is_none(),
          global_alloc(g_name, g_alloc);

        alloc_points_to(
            Arc::new(Alloc::Global(g_alloc.clone())),
            stdout_alloc.clone()) <--
          for (g_name, g) in &module.globals,
          if **g_name == GlobalName::from("stdout"),
          if g.initializer.is_none(),
          global_alloc(g_name, g_alloc);


        alloc_points_to(ctype_b_loc_alloc, ctype_b_loc_alloc_alloc) <-- if true;

        // ----------------------------------------------------------
        // Signatures
        // ----------------------------------------------------------

        // https://galoisinc.github.io/MATE/signatures.html

        relation needs_signature(UArc<FunctionName>);

        needs_signature(name) <--
          for (name, decl) in &module.decls,
          if !name.starts_with("llvm.memcpy") &&
             !name.starts_with("llvm.memmove"),
          if !known_functions.contains(name.as_ref()),
          if sigs.get(name).is_none(),
          if decl.has_pointer();

        // Functions without signatures must be treated conservatively
        operand_points_to(ctx, call_name.operand(), top.clone()) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "operand_points_to"
          } else {
              "extern_top"
          }),
          let _span = span.enter(),
          //
          calls(ctx, call_name, callee_name, _, _),
          needs_signature(callee_name),
          if let Some(decl) = module.decls.get(callee_name),
          if let llvm_ir::Type::PointerType{ .. } = &*decl.return_type,
          //
          if count("operand_points_to", "extern_top");

        operand_points_to(ctx, call_name.operand(), a) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "operand_points_to"
          } else {
              "sig_return_alloc"
          }),
          let _span = span.enter(),
          //
          calls(ctx, call_name, callee_name, _, _),
          if let Some(s) = sigs.get(callee_name),
          for sig in s,
          if let Signature::ReturnAlloc { r#type } = sig,
          let a = match r#type {
              AllocType::Heap => Arc::new(Alloc::Heap(
                  HeapAlloc::new(call_name.instruction_name(), None))
              ),
              AllocType::Stack => Arc::new(Alloc::Stack(
                  StackAlloc::signature(call_name.instruction_name()))
              ),
              AllocType::Top => top.clone(),
          },
          //
          if count("operand_points_to", "sig_return_alloc");

        operand_points_to(ctx, call_name.operand(), a) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "operand_points_to"
          } else {
              "sig_return_aliases_arg"
          }),
          let _span = span.enter(),
          //
          calls(ctx, call_name, callee_name, args, _),
          if let Some(s) = sigs.get(callee_name),
          for sig in s,
          if let Signature::ReturnAliasesArg { arg } = sig,
          if let Some(op) = args.get(*arg),
          operand_points_to(ctx, op, a),
          //
          if count("operand_points_to", "sig_return_aliases_arg");

        operand_points_to(
            ctx,
            call_name.operand(),
            Arc::new(Alloc::Global(alloc_name.clone()))) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "operand_points_to"
          } else {
              "sig_return_points_to_global"
          }),
          let _span = span.enter(),
          //
          calls(ctx, call_name, callee_name, _, _),
          if let Some(s) = sigs.get(callee_name),
          for sig in s,
          if let Signature::ReturnPointsToGlobal { global } = sig,
          let global_name = Arc::new(GlobalName::from(global.as_ref())),
          global_alloc(global_name, alloc_name),
          //
          if count("operand_points_to", "sig_return_points_to_global");

        operand_points_to(
            ctx,
            call_name.operand(),
            Arc::new(Alloc::Global(alloc_name))) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "operand_points_to"
          } else {
              "sig_return_points_to_global_fresh"
          }),
          let _span = span.enter(),
          //
          calls(ctx, call_name, callee_name, _, _),
          if let Some(s) = sigs.get(callee_name),
          for sig in s,
          if let Signature::ReturnPointsToGlobal { global } = sig,
          let global_name = Arc::new(GlobalName::from(global.as_ref())),
          !global_alloc(global_name, _),
          let alloc_name = GlobalAlloc::new(global_name.clone(), false, None),
          //
          if count("operand_points_to", "sig_return_points_to_global_fresh");

        memcpy(ctx, dst_op.clone(), src_op.clone(), None) <--
          let span = trace_span!(if cfg!(feature = "relation") {
              "memcpy"
          } else {
              "sig_arg_memcpy_arg"
          }),
          let _span = span.enter(),
          //
          calls(ctx, call_name, callee_name, args, _),
          if let Some(s) = sigs.get(callee_name),
          for sig in s,
          if let Signature::ArgMemcpyArg { dst, src } = sig,
          if dst != src,
          if let Some(dst_op) = args.get(*dst),
          if let Some(src_op) = args.get(*src),
          //
          if count("memcpy", "sig_arg_memcpy_arg");

        // ----------------------------------------------------------
        // Assertions
        // ----------------------------------------------------------

        // This can also fail right now due to incomplete coverage of LLVM
        // features generally (e.g., exceptions, returnaddress/stacksave/
        // stackrestore). See xfail tests. Need to implement a generic "top"/
        // unknown result for complex features like exceptions.
        relation _assert_reachable_pointers_point_to_something();
        _assert_reachable_pointers_point_to_something() <--
          if opts.check_assertions && opts.check_strict,
          // A function lacking a signature has unknown effects on the points-
          // to graph, invalidating this assertion.
          !needs_signature(_),
          function_instruction_opcode(f, i, o, t),
          reachable(ctx, f),
          if let llvm_ir::Type::PointerType{ .. } = **t,
          // See NOTE[pass-thru]
          if !matches!(
              o.as_ref(),
              Opcode::BitCast(_) |
                Opcode::GetElementPtr(_) |
                Opcode::IntToPtr(_) |
                Opcode::PtrToInt(_)
          ),
          !operand_points_to(ctx, i.operand(), _),
          let _ = panic!("Bug! Instruction doesn't point to anything: {}", i.instruction_name());

        relation _assert_bitcast_constants_point_to_something();
        _assert_bitcast_constants_point_to_something() <--
          if opts.check_assertions,
          reachable_instruction_opcode!(ctx, instr, opcode),
          for op in &opcode.as_ref().operands(),
          if let Operand::Constant(ref const_arc) = &**op,
          if let Constant::BitCast(_) = &**const_arc,
          !operand_points_to(ctx, op, _),
          let _ = panic!("Bug! bitcast expression doesn't point to anything: {op}");

        relation _assert_gep_constants_point_to_something();
        _assert_gep_constants_point_to_something() <--
          if opts.check_assertions,
          reachable_instruction_opcode!(ctx, instr, opcode),
          for op in &opcode.as_ref().operands(),
          if let Operand::Constant(ref const_arc) = &**op,
          if let Constant::GetElementPtr(_) = &**const_arc,
          !operand_points_to(ctx, op, _),
          let _ = panic!("Bug! gep expression doesn't point to anything: {op}");

        // ----------------------------------------------------------
        // Precision metrics
        // ----------------------------------------------------------

        // See comments on `Metrics`.

        relation callgraph(UArc<InstructionName>, UArc<FunctionName>);
        callgraph(i.instruction_name(), f) <-- calls(_, i, f, _, _);

        relation free_non_heap(Arc<Alloc>);
        free_non_heap(a.clone()) <--
          calls(ctx, i, f, args, _),
          if **f == "free",
          if let Some(ptr) = args.get(0),
          operand_points_to(ctx, ptr, a),
          if !a.freeable();

        relation invalid_call(Arc<Operand>, Arc<Alloc>);
        invalid_call(pointer.clone(), alloc.clone()) <--
          if opts.metrics,
          call(instr, pointer, _),
          operand_points_to(ctx, pointer, alloc),
          if !matches!(&**alloc, Alloc::Function(_));

        relation invalid_load(Arc<Operand>, Arc<Alloc>);
        invalid_load(pointer.clone(), alloc.clone()) <--
          if opts.metrics,
          load(instr, pointer),
          operand_points_to(_ctx, pointer, alloc),
          if !alloc.loadable();

        relation invalid_memcpy_dst(Arc<Operand>, Arc<Alloc>);
        invalid_memcpy_dst(dst.clone(), dst_alloc.clone()) <--
          if opts.metrics,
          memcpy(ctx, dst, _, sz),
          let min_size = sz.unwrap_or(0),
          operand_points_to(ctx, dst, dst_alloc),
          if !dst_alloc.storable();

        relation invalid_memcpy_src(Arc<Operand>, Arc<Alloc>);
        invalid_memcpy_src(src.clone(), src_alloc.clone()) <--
          if opts.metrics,
          memcpy(ctx, _, src, sz),
          let min_size = sz.unwrap_or(0),
          operand_points_to(ctx, src, src_alloc),
          if !src_alloc.loadable();

        relation invalid_store(Arc<Operand>, Arc<Alloc>);
        invalid_store(pointer.clone(), alloc.clone()) <--
          if opts.metrics,
          store(instr, pointer, _),
          operand_points_to(_ctx, pointer, alloc),
          if !alloc.storable();

        relation points_to_top(Arc<Operand>);
        points_to_top(op.clone()) <--
          operand_points_to(_, op, alloc),
          if let Alloc::Top = &**alloc;
    };

    if opts.debug {
        eprintln!("{}", outs.summary());
        eprintln!("{}", outs.scc_times_summary());
    }

    OutputRelations {
        alloc_points_to: outs
            .alloc_points_to
            .into_iter()
            .map(|(i, a)| (i, Alloc::lookup(&a)))
            .collect(),
        operand_points_to: outs
            .operand_points_to
            .into_iter()
            .map(|(c, i, a)| (c, i, Alloc::lookup(&a)))
            .collect(),
        reachable: outs.reachable.into_iter().map(|tup| tup.1).collect(),
        calls: outs
            .calls
            .into_iter()
            .map(|tup| ((tup.0, tup.1.instruction_name()), tup.2))
            .collect(),
        needs_signature: outs.needs_signature.into_iter().map(|tup| tup.0).collect(),
        metrics: if opts.metrics {
            Some(Metrics {
                callgraph_size: outs.callgraph.len(),
                free_non_heap: outs.free_non_heap.len(),
                invalid_calls: outs.invalid_call.len(),
                invalid_loads: outs.invalid_load.len(),
                invalid_memcpy_dsts: outs.invalid_memcpy_src.len(),
                invalid_memcpy_srcs: outs.invalid_memcpy_dst.len(),
                invalid_stores: outs.invalid_store.len(),
                points_to_top: outs.points_to_top.len(),
            })
        } else {
            None
        },
    }
}
