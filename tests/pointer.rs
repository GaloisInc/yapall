// To debug or develop a test, try `eprintln!("{:#?}", out)`

// TODO: Run each test at different levels of context sensitivity!

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    process::Command,
};

use yapall::{
    analysis::pointer,
    llvm::constant::Constant,
    llvm::instruction::{Call, Opcode},
    pointer::Options,
    pointer::OutputRelations,
    Alloc, Arc, Callee, FunctionName, Invoke, Module, Operand, Signatures, TerminatorOpcode, UArc,
};

// ------------------------------------------------------------------
// Helpers

fn rustc() -> Option<String> {
    match std::env::var("RUSTC_LLVM_14") {
        Ok(s) => Some(s),
        Err(_) => None,
    }
}

fn run_rustc(prog: &str, dir: &str, out: &str, opt: u8) -> bool {
    let compiler = rustc().unwrap_or_else(|| "rustc".to_string());
    let status = Command::new(compiler)
        .arg("--emit=llvm-bc")
        .arg("-C")
        .arg(format!("opt-level={}", opt))
        .arg("-o")
        .arg(&out)
        .arg(format!("tests/{}/{}", dir, prog))
        .status()
        .unwrap();
    status.success()
}

fn compile(prog: &str, dir: &str, opt: u8) -> llvm_ir::Module {
    let out = format!("tests/{}/{}-{}.bc", dir, prog, opt);
    if prog.ends_with(".c") || prog.ends_with(".cpp") {
        let compiler = if prog.ends_with(".c") {
            "clang"
        } else {
            "clang++"
        };
        let status = Command::new(compiler)
            .arg("-emit-llvm")
            .arg("-c")
            .arg(format!("-O{}", opt))
            .arg("-Werror")
            .arg("-o")
            .arg(&out)
            .arg(format!("tests/{}/{}", dir, prog))
            .status()
            .unwrap();
        assert!(status.success());
        // if prog.ends_with(".cpp") {
        //     let status = Command::new("llvm-link")
        //         .arg("--only-needed")
        //         .arg(&out)
        //         .arg("libcxx-7.1.0.bc")
        //         .arg("-o")
        //         .arg(&out)
        //         .status()
        //         .unwrap();
        //     assert!(status.success());
        // }
    } else if prog.ends_with(".rs") {
        // Work around sad non-determinism :-(
        for i in 0..4 {
            std::thread::sleep(std::time::Duration::new(1, 0));
            let success = run_rustc(prog, dir, &out, opt);
            if success {
                break;
            }
            assert!(i != 4);
        }
    } else {
        panic!("Bad program path, expected .c or .cpp: {prog}");
    };
    let path = PathBuf::from(&out);
    llvm_ir::Module::from_bc_path(path).unwrap()
}

fn points_to_something(out: &OutputRelations, operand: &Operand) -> bool {
    for (_, op, _alloc) in &out.operand_points_to {
        if **op == *operand {
            return true;
        }
    }
    false
}

fn points_to<'a>(out: &'a OutputRelations, op: &Operand) -> HashSet<&'a Alloc> {
    let mut points_to: HashSet<&Alloc> = HashSet::new();
    for (_, o, alloc) in &out.operand_points_to {
        if **o == *op {
            points_to.insert(alloc);
        }
    }
    points_to
}

fn disjoint(out: &OutputRelations, args: &Vec<Arc<Operand>>) -> bool {
    assert!(args.len() == 2);
    let points_to_0: HashSet<&Alloc> = points_to(out, &args[0]);
    let points_to_1: HashSet<&Alloc> = points_to(out, &args[1]);
    points_to_0.is_disjoint(&points_to_1)
}

fn check_call(
    out: &OutputRelations,
    caller: &UArc<FunctionName>,
    callee: &Callee,
    arguments: &Vec<Arc<Operand>>,
) {
    if let Callee::Operand(op) = &callee {
        if let Operand::Constant(const_arc) = &**op {
            if let Constant::Function(name) = &**const_arc {
                if **name == FunctionName::from("assert_points_to_nothing") {
                    for arg in arguments {
                        assert!(!points_to_something(&out, arg));
                    }
                } else if **name == FunctionName::from("assert_points_to_something") {
                    for arg in arguments {
                        assert!(points_to_something(&out, arg));
                    }
                } else if **name == FunctionName::from("assert_reachable") {
                    assert!(out.reachable.contains(caller));
                } else if **name == FunctionName::from("assert_may_alias") {
                    assert!(!disjoint(&out, arguments));
                } else if **name == FunctionName::from("assert_disjoint") {
                    assert!(disjoint(&out, arguments));
                } else if **name == FunctionName::from("assert_unreachable") {
                    assert!(!out.reachable.contains(caller));
                } else if (**name).to_string().starts_with("@assert_") {
                    panic!("Unknown assertion: {name}")
                }
            }
        }
    }
}

fn check_module(out: &OutputRelations, module: &Module) {
    for (f_name, f) in &module.functions {
        for b in &f.blocks {
            if let TerminatorOpcode::Invoke(Invoke { callee, args, .. }) =
                &*b.terminator.opcode.as_ref()
            {
                check_call(&out, &f_name, &callee, args);
            }
            for i in &b.instrs {
                if let Opcode::Call(Call { callee, args, .. }) = &*i.opcode.as_ref() {
                    check_call(&out, &f_name, &callee, args);
                }
            }
        }
    }
}

fn signatures(program: &str, dir: &str) -> Signatures {
    let path_str = format!("tests/{}/{}", dir, program);
    let path = Path::new(&path_str);
    let signatures_path = path.with_extension("json");
    if signatures_path.try_exists().unwrap() {
        let signatures_string =
            std::fs::read_to_string(signatures_path).expect("Couldn't read points-to signautres");
        Signatures::new(
            serde_json::from_str(&signatures_string)
                .expect("Couldn't deserialize points-to signatures"),
        )
        .unwrap()
    } else {
        Signatures::default()
    }
}

struct NamedModule {
    dir: String,
    program: String,
    module: Module,
}

fn convert(program: &str, dir: &str, opt: u8) -> NamedModule {
    let llvm_module = compile(program, dir, opt);
    let mut operands: HashMap<Arc<Operand>, &llvm_ir::Operand> = HashMap::new();
    match Module::new(&llvm_module, &mut operands) {
        Ok(m) => NamedModule {
            dir: dir.to_owned(),
            program: program.to_owned(),
            module: m,
        },
        Err(e) => panic!("{}", e),
    }
}

fn check(module: &NamedModule) -> OutputRelations {
    let sigs = signatures(&module.program, &module.dir);
    let opts = Options {
        check_assertions: true,
        // This program intentionally constructs a nonsense pointer, triggering
        // an assertion failure before the test can complete.
        check_strict: module.program != "fail-assert-points-to-something.c",
        contexts: 1,
        debug: false,
        metrics: true,
        unification: true,
    };
    let out = pointer::analysis(&module.module, &sigs, &opts);
    check_module(&out, &module.module);
    out
}

fn imprecise(program: &str, opt: u8) -> NamedModule {
    convert(program, "pointer/imprecision", opt)
}

fn precise(program: &str, opt: u8) -> NamedModule {
    convert(program, "pointer/precision", opt)
}

fn property(program: &str, opt: u8) -> NamedModule {
    convert(program, "property", opt)
}

fn signature(program: &str, opt: u8) -> NamedModule {
    convert(program, "pointer/signatures", opt)
}

fn sound(program: &str, opt: u8) -> NamedModule {
    convert(program, "pointer/soundness", opt)
}

fn template(program: &str, opt: u8) -> NamedModule {
    convert(program, "pointer/templates", opt)
}

// ------------------------------------------------------------------

// TODO(#48): Fix me!
// #[test]
// fn irving_precision() {
//     let module = convert("irving.c", "medium", 1);
//     if let Some(m) = check(&module).metrics {
//         assert_eq!(m.callgraph_size, 803);
//         assert_eq!(m.free_non_heap, 4);
//         assert_eq!(m.invalid_loads, 138);
//         assert_eq!(m.invalid_stores, 17);
//     }
// }

#[test]
#[ignore]
fn properties() {
    for program in std::fs::read_dir("tests/property").unwrap() {
        let path = program.as_ref().unwrap().path();
        println!("{}", path.to_string_lossy());
        if path.to_string_lossy().ends_with(".c") {
            // || path.to_string_lossy().ends_with(".cpp") {
            let module = property(
                path.strip_prefix("tests/property")
                    .unwrap()
                    .to_str()
                    .unwrap(),
                1,
            );
            let _out = check(&module);
        }
    }
}

#[test]
fn alloca_o0() {
    let module = sound("alloca.c", 0);
    let _out = check(&module);
}

#[test]
fn alloca_o1() {
    let module = sound("alloca.c", 1);
    let _out = check(&module);
}

#[test]
fn alloca_o2() {
    let module = sound("alloca.c", 2);
    let _out = check(&module);
}

#[test]
fn argv_o0() {
    let module = sound("argv.c", 0);
    let _out = check(&module);
}

#[test]
fn argv_o1() {
    let module = sound("argv.c", 1);
    let _out = check(&module);
}

#[test]
fn argv_o2() {
    let module = sound("argv.c", 2);
    let _out = check(&module);
}

#[test]
fn argv_0_o0() {
    let module = sound("argv-0.c", 0);
    let _out = check(&module);
}

#[test]
fn argv_0_o1() {
    let module = sound("argv-0.c", 1);
    let _out = check(&module);
}

#[test]
fn argv_0_o2() {
    let module = sound("argv-0.c", 2);
    let _out = check(&module);
}

#[test]
fn r#array_o0() {
    let module = imprecise("array.c", 0);
    let _out = check(&module);
}

#[test]
fn r#array_o1() {
    let module = imprecise("array.c", 1);
    let _out = check(&module);
}

#[test]
fn r#array_o2() {
    let module = imprecise("array.c", 2);
    let _out = check(&module);
}

#[test]
fn call_o0() {
    let module = sound("call.c", 0);
    let out = check(&module);
    // Make sure that callee wasn't inlined:
    assert!(out.reachable.len() == 3); // main, callee, printf
}

#[test]
fn call_o1() {
    let module = sound("call.c", 1);
    let out = check(&module);
    // Make sure that callee wasn't inlined:
    assert!(out.reachable.len() == 3); // main, callee, printf
}

#[test]
fn call_o2() {
    let module = sound("call.c", 2);
    let out = check(&module);
    // Make sure that callee wasn't inlined:
    assert!(out.reachable.len() == 3); // main, callee, printf
}

#[test]
fn call_alloca_o0() {
    let module = sound("call-alloca.c", 1);
    let _out = check(&module);
}

#[test]
fn call_alloca_o1() {
    let module = sound("call-alloca.c", 1);
    let _out = check(&module);
}

#[test]
fn call_alloca_o2() {
    let module = sound("call-alloca.c", 2);
    let _out = check(&module);
}

#[test]
fn calloc_o0() {
    let module = sound("calloc.c", 0);
    let _out = check(&module);
}

#[test]
fn calloc_o1() {
    let module = sound("calloc.c", 1);
    let _out = check(&module);
}

#[test]
fn calloc_o2() {
    let module = sound("calloc.c", 2);
    let _out = check(&module);
}

#[test]
fn context_o0() {
    let module = imprecise("context.c", 0);
    let _out = check(&module);
}

// XFAIL: Yay, context-sensitivity!
#[test]
#[should_panic(expected = "assertion failed: !disjoint")]
fn context_o1() {
    let module = imprecise("context.c", 1);
    let _out = check(&module);
}

// XFAIL: Yay, context-sensitivity!
#[test]
#[should_panic(expected = "assertion failed: !disjoint")]
fn context_o2() {
    let module = imprecise("context.c", 2);
    let _out = check(&module);
}

// XFAIL: This proves that this assertion is being handled properly by tests
#[test]
#[should_panic(expected = "assertion failed: disjoint")]
fn fail_assert_disjoint_o1() {
    let module = precise("fail-assert-disjoint.c", 1);
    let _out = check(&module);
}

#[test]
#[should_panic(expected = "assertion failed: disjoint")]
fn fail_assert_disjoint_o2() {
    let module = precise("fail-assert-disjoint.c", 2);
    let _out = check(&module);
}

// XFAIL: This proves that this assertion is being handled properly by tests
#[test]
#[should_panic(expected = "assertion failed: !disjoint")]
fn fail_assert_may_alias_o1() {
    let module = imprecise("fail-assert-may-alias.c", 1);
    let _out = check(&module);
}

#[test]
#[should_panic(expected = "assertion failed: !disjoint")]
fn fail_assert_may_alias_o2() {
    let module = imprecise("fail-assert-may-alias.c", 2);
    let _out = check(&module);
}

// XFAIL: This proves that this assertion is being handled properly by tests
#[test]
#[should_panic(expected = "assertion failed: out.reachable.contains")]
fn fail_assert_reachable_o1() {
    let module = sound("fail-assert-reachable.c", 1);
    let _out = check(&module);
}

#[test]
#[should_panic(expected = "assertion failed: out.reachable.contains")]
fn fail_assert_reachable_o2() {
    let module = sound("fail-assert-reachable.c", 2);
    let _out = check(&module);
}

// XFAIL: This proves that this assertion is being handled properly by tests
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn fail_assert_points_to_something_o1() {
    let module = sound("fail-assert-points-to-something.c", 1);
    let _out = check(&module);
}

#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn fail_assert_points_to_something_o2() {
    let module = sound("fail-assert-points-to-something.c", 2);
    let _out = check(&module);
}

// XFAIL: This proves that this assertion is being handled properly by tests
#[test]
#[should_panic(expected = "assertion failed: !out.reachable.contains")]
fn fail_assert_unreachable_o1() {
    let module = precise("fail-assert-unreachable.c", 1);
    let _out = check(&module);
}

#[test]
#[should_panic(expected = "assertion failed: !out.reachable.contains")]
fn fail_assert_unreachable_o2() {
    let module = precise("fail-assert-unreachable.c", 2);
    let _out = check(&module);
}

#[test]
fn func_ptr_o0() {
    let module = sound("func-ptr.c", 0);
    let _out = check(&module);
}

#[test]
fn func_ptr_o1() {
    let module = sound("func-ptr.c", 1);
    let _out = check(&module);
}

#[test]
fn func_ptr_o2() {
    let module = sound("func-ptr.c", 2);
    let _out = check(&module);
}

#[test]
fn function_table_o0() {
    let module = sound("function-table.c", 0);
    let _out = check(&module);
}

#[test]
fn function_table_o1() {
    let module = sound("function-table.c", 1);
    let _out = check(&module);
}

#[test]
fn function_table_o2() {
    let module = sound("function-table.c", 2);
    let _out = check(&module);
}

#[test]
fn gep_o0() {
    let module = sound("gep.c", 0);
    let _out = check(&module);
}

#[test]
fn gep_o1() {
    let module = sound("gep.c", 1);
    let _out = check(&module);
}

#[test]
fn gep_o2() {
    let module = sound("gep.c", 2);
    let _out = check(&module);
}

#[test]
fn global_o0() {
    let module = sound("global.c", 0);
    let _out = check(&module);
}

#[test]
fn global_o1() {
    let module = sound("global.c", 1);
    let _out = check(&module);
}

#[test]
fn global_o2() {
    let module = sound("global.c", 2);
    let _out = check(&module);
}

#[test]
fn global_array_o0() {
    let module = sound("global-array.c", 0);
    let _out = check(&module);
}

#[test]
fn global_array_o1() {
    let module = sound("global-array.c", 1);
    let _out = check(&module);
}

#[test]
fn global_array_o2() {
    let module = sound("global-array.c", 2);
    let _out = check(&module);
}

#[test]
fn global_cast_o0() {
    let module = sound("global-cast.c", 0);
    let _out = check(&module);
}

#[test]
fn global_cast_o1() {
    let module = sound("global-cast.c", 1);
    let _out = check(&module);
}

#[test]
fn global_cast_o2() {
    let module = sound("global-cast.c", 2);
    let _out = check(&module);
}

#[test]
fn global_load_o0() {
    let module = sound("global-load.c", 0);
    let _out = check(&module);
}

#[test]
fn global_load_o1() {
    let module = sound("global-load.c", 1);
    let _out = check(&module);
}

#[test]
fn global_load_o2() {
    let module = sound("global-load.c", 2);
    let _out = check(&module);
}

#[test]
fn global_store_o0() {
    let module = sound("global-store.c", 0);
    let _out = check(&module);
}

#[test]
fn global_store_o1() {
    let module = sound("global-store.c", 1);
    let _out = check(&module);
}

#[test]
fn global_store_o2() {
    let module = sound("global-store.c", 2);
    let _out = check(&module);
}

#[test]
fn global_struct_o0() {
    let module = sound("global-struct.c", 0);
    let _out = check(&module);
}

#[test]
fn global_struct_o1() {
    let module = sound("global-struct.c", 1);
    let _out = check(&module);
}

#[test]
fn global_struct_o2() {
    let module = sound("global-struct.c", 2);
    let _out = check(&module);
}

#[test]
fn global_expr_o0() {
    let module = sound("global-expr.c", 0);
    let _out = check(&module);
}

#[test]
fn global_expr_o1() {
    let module = sound("global-expr.c", 1);
    let _out = check(&module);
}

#[test]
fn global_expr_o2() {
    let module = sound("global-expr.c", 2);
    let _out = check(&module);
}

#[test]
fn heap_heap_o0() {
    let module = precise("heap-heap.c", 0);
    let _out = check(&module);
}

#[test]
fn heap_heap_o1() {
    let module = precise("heap-heap.c", 1);
    let _out = check(&module);
}

#[test]
fn heap_heap_o2() {
    let module = precise("heap-heap.c", 2);
    let _out = check(&module);
}

#[test]
fn indirect_call_o0() {
    let module = sound("indirect-call.c", 0);
    let _out = check(&module);
}

#[test]
fn indirect_call_o1() {
    let module = sound("indirect-call.c", 1);
    let _out = check(&module);
}

#[test]
fn indirect_call_o2() {
    let module = sound("indirect-call.c", 2);
    let _out = check(&module);
}

#[test]
fn main_o0() {
    let module = sound("main.c", 0);
    let _out = check(&module);
}

#[test]
fn main_o1() {
    let module = sound("main.c", 1);
    let _out = check(&module);
}

#[test]
fn main_o2() {
    let module = sound("main.c", 2);
    let _out = check(&module);
}

#[test]
fn main_rs_o0() {
    if rustc().is_none() {
        return;
    }
    let module = sound("main.rs", 0);
    let _out = check(&module);
}

#[test]
fn main_rs_o1() {
    if rustc().is_none() {
        return;
    }
    let module = sound("main.rs", 1);
    let _out = check(&module);
}

#[test]
fn main_rs_o2() {
    if rustc().is_none() {
        return;
    }
    let module = sound("main.rs", 2);
    let _out = check(&module);
}

#[test]
fn malloc_o0() {
    let module = sound("malloc.c", 0);
    let _out = check(&module);
}

#[test]
fn malloc_o1() {
    let module = sound("malloc.c", 1);
    let _out = check(&module);
}

#[test]
fn malloc_o2() {
    let module = sound("malloc.c", 2);
    let _out = check(&module);
}

#[test]
fn memcpy_o0() {
    let module = sound("memcpy.c", 0);
    let _out = check(&module);
}

#[test]
fn memcpy_o1() {
    let module = sound("memcpy.c", 1);
    let _out = check(&module);
}

#[test]
fn memcpy_o2() {
    let module = sound("memcpy.c", 2);
    let _out = check(&module);
}

#[test]
#[should_panic(expected = "assertion failed: !points_to_something")]
fn memcpy_size_o0() {
    let module = precise("memcpy-size.c", 0);
    let _out = check(&module);
}

#[test]
#[should_panic(expected = "assertion failed: !points_to_something")]
fn memcpy_size_o1() {
    let module = precise("memcpy-size.c", 1);
    let _out = check(&module);
}

#[test]
#[should_panic(expected = "assertion failed: !points_to_something")]
fn memcpy_size_o2() {
    let module = precise("memcpy-size.c", 2);
    let _out = check(&module);
}

#[test]
fn new_o0() {
    let module = sound("new.cpp", 0);
    let _out = check(&module);
}

#[test]
fn new_o1() {
    let module = sound("new.cpp", 1);
    let _out = check(&module);
}

#[test]
fn new_o2() {
    let module = sound("new.cpp", 2);
    let _out = check(&module);
}

#[test]
fn phi_o0() {
    let module = sound("phi.c", 0);
    let _out = check(&module);
}

#[test]
fn phi_o1() {
    let module = sound("phi.c", 1);
    let _out = check(&module);
}

#[test]
fn phi_o2() {
    let module = sound("phi.c", 2);
    let _out = check(&module);
}

#[test]
fn ptr_arg_o0() {
    let module = sound("ptr-arg.c", 0);
    let _out = check(&module);
}

#[test]
fn ptr_arg_o1() {
    let module = sound("ptr-arg.c", 1);
    let _out = check(&module);
}

#[test]
fn ptr_arg_o2() {
    let module = sound("ptr-arg.c", 2);
    let _out = check(&module);
}

#[test]
fn ptr_ret_o0() {
    let module = sound("ptr-ret.c", 0);
    let out = check(&module);
    // Make sure that callee wasn't inlined:
    assert!(out.reachable.len() == 4); // main, malloc, add1, assert
}

#[test]
fn ptr_ret_o1() {
    let module = sound("ptr-ret.c", 1);
    let out = check(&module);
    // Make sure that callee wasn't inlined:
    assert!(out.reachable.len() == 4); // main, malloc, add1, assert
}

#[test]
fn ptr_ret_o2() {
    let module = sound("ptr-ret.c", 2);
    let out = check(&module);
    // Make sure that callee wasn't inlined:
    assert!(out.reachable.len() == 4); // main, malloc, add1, assert
}

#[test]
fn ptr_to_int_o0() {
    let module = sound("ptr-to-int.c", 0);
    let _out = check(&module);
}

#[test]
fn ptr_to_int_o1() {
    let module = sound("ptr-to-int.c", 1);
    let _out = check(&module);
}

#[test]
fn ptr_to_int_o2() {
    let module = sound("ptr-to-int.c", 2);
    let _out = check(&module);
}

#[test]
fn ptr_to_int_sub_o0() {
    let module = sound("ptr-to-int-sub.c", 0);
    let _out = check(&module);
}

#[test]
fn ptr_to_int_sub_o1() {
    let module = sound("ptr-to-int-sub.c", 1);
    let _out = check(&module);
}

#[test]
fn ptr_to_int_sub_o2() {
    let module = sound("ptr-to-int-sub.c", 2);
    let _out = check(&module);
}

#[test]
fn realloc_o0() {
    let module = sound("realloc.c", 0);
    let _out = check(&module);
}

#[test]
fn realloc_o1() {
    let module = sound("realloc.c", 1);
    let _out = check(&module);
}

#[test]
fn realloc_o2() {
    let module = sound("realloc.c", 2);
    let _out = check(&module);
}

#[test]
fn reallocarray_o0() {
    let module = sound("reallocarray.c", 0);
    let _out = check(&module);
}

#[test]
fn reallocarray_o1() {
    let module = sound("reallocarray.c", 1);
    let _out = check(&module);
}

#[test]
fn reallocarray_o2() {
    let module = sound("reallocarray.c", 2);
    let _out = check(&module);
}

#[test]
fn template_shared_ptr_o0() {
    let module = template("shared-ptr.cpp", 0);
    let _out = check(&module);
}

#[test]
fn template_shared_ptr_o1() {
    let module = template("shared-ptr.cpp", 1);
    let _out = check(&module);
}

#[test]
fn template_shared_ptr_o2() {
    let module = template("shared-ptr.cpp", 2);
    let _out = check(&module);
}

// XFAIL(TODO)
#[test]
#[should_panic(expected = "Bug!")]
fn sig_arg_memcpy_arg_o0() {
    let module = signature("arg-memcpy-arg.c", 0);
    let _out = check(&module);
}

#[test]
fn sig_arg_memcpy_arg_o1() {
    let module = signature("arg-memcpy-arg.c", 1);
    let _out = check(&module);
}

#[test]
fn sig_arg_memcpy_arg_o2() {
    let module = signature("arg-memcpy-arg.c", 2);
    let _out = check(&module);
}

#[test]
fn sig_return_alloc_o0() {
    let module = signature("return-alloc.c", 0);
    let _out = check(&module);
}

#[test]
fn sig_return_alloc_o1() {
    let module = signature("return-alloc.c", 1);
    let _out = check(&module);
}

#[test]
fn sig_return_alloc_o2() {
    let module = signature("return-alloc.c", 2);
    let _out = check(&module);
}

#[test]
fn sig_return_aliases_arg_o0() {
    let module = signature("return-aliases-arg.c", 0);
    let _out = check(&module);
}

#[test]
fn sig_return_aliases_arg_o1() {
    let module = signature("return-aliases-arg.c", 1);
    let _out = check(&module);
}

#[test]
fn sig_return_aliases_arg_o2() {
    let module = signature("return-aliases-arg.c", 2);
    let _out = check(&module);
}

#[test]
fn sig_return_points_to_global_o0() {
    let module = signature("return-points-to-global.c", 0);
    let _out = check(&module);
}

#[test]
fn sig_return_points_to_global_o1() {
    let module = signature("return-points-to-global.c", 1);
    let _out = check(&module);
}

#[test]
fn sig_return_points_to_global_o2() {
    let module = signature("return-points-to-global.c", 2);
    let _out = check(&module);
}

#[test]
fn sig_sig_o0() {
    let module = precise("sig-sig.c", 0);
    let _out = check(&module);
}

#[test]
fn sig_sig_o1() {
    let module = precise("sig-sig.c", 1);
    let _out = check(&module);
}

#[test]
fn sig_sig_o2() {
    let module = precise("sig-sig.c", 2);
    let _out = check(&module);
}

#[test]
fn slice_o0() {
    let module = sound("slice.rs", 0);
    let _out = check(&module);
}

#[test]
fn slice_o1() {
    let module = sound("slice.rs", 1);
    let _out = check(&module);
}

#[test]
fn slice_o2() {
    let module = sound("slice.rs", 2);
    let _out = check(&module);
}

#[test]
fn stack_o0() {
    let module = sound("stack.rs", 0);
    let _out = check(&module);
}

#[test]
fn stack_o1() {
    let module = sound("stack.rs", 1);
    let _out = check(&module);
}

#[test]
fn stack_o2() {
    let module = sound("stack.rs", 2);
    let _out = check(&module);
}

#[test]
fn stack_cast_o0() {
    let module = sound("stack-cast.c", 0);
    let _out = check(&module);
}

#[test]
fn stack_cast_o1() {
    let module = sound("stack-cast.c", 1);
    let _out = check(&module);
}

#[test]
fn stack_cast_o2() {
    let module = sound("stack-cast.c", 2);
    let _out = check(&module);
}

#[test]
fn stack_stack_o0() {
    let module = precise("stack-stack.c", 0);
    let _out = check(&module);
}

#[test]
fn stack_stack_o1() {
    let module = precise("stack-stack.c", 1);
    let _out = check(&module);
}

#[test]
fn stack_stack_o2() {
    let module = precise("stack-stack.c", 2);
    let _out = check(&module);
}

#[test]
fn stack_array_o0() {
    let module = sound("stack-struct.c", 0);
    let _out = check(&module);
}

#[test]
fn stack_array_o1() {
    let module = sound("stack-struct.c", 1);
    let _out = check(&module);
}

#[test]
fn stack_array_o2() {
    let module = sound("stack-struct.c", 2);
    let _out = check(&module);
}

#[test]
fn stack_struct_o0() {
    let module = sound("stack-struct.c", 0);
    let _out = check(&module);
}

#[test]
fn stack_struct_o1() {
    let module = sound("stack-struct.c", 1);
    let _out = check(&module);
}

#[test]
fn stack_struct_o2() {
    let module = sound("stack-struct.c", 2);
    let _out = check(&module);
}

#[test]
fn stderr_o0() {
    let module = sound("stderr.c", 0);
    let _out = check(&module);
}

#[test]
fn stderr_o1() {
    let module = sound("stderr.c", 1);
    let _out = check(&module);
}

#[test]
fn stderr_o2() {
    let module = sound("stderr.c", 2);
    let _out = check(&module);
}

#[test]
fn str_o0() {
    let module = sound("str.rs", 0);
    let _out = check(&module);
}

#[test]
fn str_o1() {
    let module = sound("str.rs", 1);
    let _out = check(&module);
}

#[test]
fn str_o2() {
    let module = sound("str.rs", 2);
    let _out = check(&module);
}

// XFAIL(TODO)
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn string_o0() {
    let module = sound("string.rs", 0);
    let _out = check(&module);
}

// XFAIL(TODO)
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn string_o1() {
    let module = sound("string.rs", 1);
    let _out = check(&module);
}

// XFAIL(TODO)
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn string_o2() {
    let module = sound("string.rs", 2);
    let _out = check(&module);
}

#[test]
fn r#struct_o0() {
    let module = imprecise("struct.c", 0);
    let _out = check(&module);
}

#[test]
fn r#struct_o1() {
    let module = imprecise("struct.c", 1);
    let _out = check(&module);
}

#[test]
fn r#struct_o2() {
    let module = imprecise("struct.c", 2);
    let _out = check(&module);
}

// XFAIL(TODO): This started failing when implementing context sensitivity.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_any_o0() {
    if rustc().is_none() {
        return;
    }
    let module = template("any.rs", 0);
    let _out = check(&module);
}

#[test]
fn template_any_o1() {
    if rustc().is_none() {
        return;
    }
    let module = template("any.rs", 1);
    let _out = check(&module);
}

#[test]
fn template_any_o2() {
    if rustc().is_none() {
        return;
    }
    let module = template("any.rs", 2);
    let _out = check(&module);
}

#[test]
fn template_array_o0() {
    let module = template("array.cpp", 0);
    let _out = check(&module);
}

#[test]
fn template_array_o1() {
    let module = template("array.cpp", 1);
    let _out = check(&module);
}

#[test]
fn template_array_o2() {
    let module = template("array.cpp", 2);
    let _out = check(&module);
}

// XFAIL(TODO): This started failing when instruction_points_to was made
// conditional on reachability of the containing function.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_box_o0() {
    if rustc().is_none() {
        return;
    }
    let module = template("box.rs", 0);
    let _out = check(&module);
}

#[test]
fn template_box_o1() {
    if rustc().is_none() {
        return;
    }
    let module = template("box.rs", 1);
    let _out = check(&module);
}

#[test]
fn template_box_o2() {
    if rustc().is_none() {
        return;
    }
    let module = template("box.rs", 2);
    let _out = check(&module);
}

#[test]
fn template_cell_o0() {
    if rustc().is_none() {
        return;
    }
    let module = template("cell.rs", 0);
    let _out = check(&module);
}

#[test]
fn template_cell_o1() {
    if rustc().is_none() {
        return;
    }
    let module = template("cell.rs", 1);
    let _out = check(&module);
}

#[test]
fn template_cell_o2() {
    if rustc().is_none() {
        return;
    }
    let module = template("cell.rs", 2);
    let _out = check(&module);
}

// XFAIL(TODO): This started failing when instruction_points_to was made
// conditional on reachability of the containing function.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_list_o0() {
    let module = template("list.cpp", 0);
    let _out = check(&module);
}

// XFAIL(TODO): This started failing when implementing context sensitivity.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_list_o1() {
    let module = template("list.cpp", 1);
    let _out = check(&module);
}

#[test]
fn template_list_o2() {
    let module = template("list.cpp", 2);
    let _out = check(&module);
}

// XFAIL(TODO): This started failing when instruction_points_to was made
// conditional on reachability of the containing function.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_queue_o0() {
    let module = template("queue.cpp", 0);
    let _out = check(&module);
}

// XFAIL(TODO): This started failing when instruction_points_to was made
// conditional on reachability of the containing function.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_queue_o1() {
    let module = template("queue.cpp", 1);
    let _out = check(&module);
}

#[test]
fn template_queue_o2() {
    let module = template("queue.cpp", 2);
    let _out = check(&module);
}

// XFAIL(TODO): This started failing when instruction_points_to was made
// conditional on reachability of the containing function.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_rc_o0() {
    if rustc().is_none() {
        return;
    }
    let module = template("rc.rs", 0);
    let _out = check(&module);
}

#[test]
fn template_rc_o1() {
    if rustc().is_none() {
        return;
    }
    let module = template("rc.rs", 1);
    let _out = check(&module);
}

#[test]
fn template_rc_o2() {
    if rustc().is_none() {
        return;
    }
    let module = template("rc.rs", 2);
    let _out = check(&module);
}

#[test]
fn template_refcell_o0() {
    if rustc().is_none() {
        return;
    }
    let module = template("refcell.rs", 0);
    let _out = check(&module);
}

#[test]
fn template_refcell_o1() {
    if rustc().is_none() {
        return;
    }
    let module = template("refcell.rs", 1);
    let _out = check(&module);
}

#[test]
fn template_refcell_o2() {
    if rustc().is_none() {
        return;
    }
    let module = template("refcell.rs", 2);
    let _out = check(&module);
}

// TODO: These make the test binary exit with SIGILL...?

// #[test]
// fn template_arc_o0() {
//     if rustc().is_none() {
//         return;
//     }
//     let module = template("arc.rs", 0);
//     let _out = check(&module);
// }

// #[test]
// fn template_arc_o1() {
//     if rustc().is_none() {
//         return;
//     }
//     let module = template("arc.rs", 1);
//     let _out = check(&module);
// }

// #[test]
// fn template_arc_o2() {
//     if rustc().is_none() {
//         return;
//     }
//     let module = template("arc.rs", 2);
//     let _out = check(&module);
// }

// XFAIL(TODO): This started failing when instruction_points_to was made
// conditional on reachability of the containing function.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_stack_o0() {
    let module = template("stack.cpp", 0);
    let _out = check(&module);
}

// XFAIL(TODO): This started failing when instruction_points_to was made
// conditional on reachability of the containing function.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_stack_o1() {
    let module = template("stack.cpp", 1);
    let _out = check(&module);
}

#[test]
fn template_stack_o2() {
    let module = template("stack.cpp", 2);
    let _out = check(&module);
}

// XFAIL(TODO): This started failing when instruction_points_to was made
// conditional on reachability of the containing function.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_unique_ptr_o0() {
    let module = template("unique-ptr.cpp", 0);
    let _out = check(&module);
}

#[test]
fn template_unique_ptr_o1() {
    let module = template("unique-ptr.cpp", 1);
    let _out = check(&module);
}

#[test]
fn template_unique_ptr_o2() {
    let module = template("unique-ptr.cpp", 2);
    let _out = check(&module);
}

// XFAIL(TODO)
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_vec_o0() {
    if rustc().is_none() {
        return;
    }
    let module = template("vec.rs", 0);
    let _out = check(&module);
}

// XFAIL(TODO): This started failing when instruction_points_to was made
// conditional on reachability of the containing function.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_vec_o1() {
    if rustc().is_none() {
        return;
    }
    let module = template("vec.rs", 1);
    let _out = check(&module);
}

// XFAIL(TODO): This started failing when instruction_points_to was made
// conditional on reachability of the containing function.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_vector_o0() {
    let module = template("vector.cpp", 0);
    let _out = check(&module);
}

// XFAIL(TODO): This started failing when instruction_points_to was made
// conditional on reachability of the containing function.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_vector_o1() {
    let module = template("vector.cpp", 1);
    let _out = check(&module);
}

#[test]
fn template_vector_o2() {
    let module = template("vector.cpp", 2);
    let _out = check(&module);
}

// XFAIL(TODO): This started failing when instruction_points_to was made
// conditional on reachability of the containing function.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_vector_push_back_o0() {
    let module = template("vector-push-back.cpp", 0);
    let _out = check(&module);
}

// XFAIL(TODO): This started failing when implementing context sensitivity.
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn template_vector_push_back_o1() {
    let module = template("vector-push-back.cpp", 1);
    let _out = check(&module);
}

#[test]
fn template_vector_push_back_o2() {
    let module = template("vector-push-back.cpp", 2);
    let _out = check(&module);
}

#[test]
fn throw_o1() {
    let module = sound("throw.cpp", 1);
    let _out = check(&module);
}

#[test]
fn throw_o2() {
    let module = sound("throw.cpp", 2);
    let _out = check(&module);
}

#[test]
fn undef_o0() {
    let module = sound("undef.c", 0);
    let _out = check(&module);
}

#[test]
fn undef_o1() {
    let module = sound("undef.c", 1);
    let _out = check(&module);
}

#[test]
fn undef_o2() {
    let module = sound("undef.c", 2);
    let _out = check(&module);
}

// XFAIL(TODO): Can't handle exceptions yet
#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn varargs_o1() {
    let module = sound("varargs.c", 1);
    let _out = check(&module);
}

#[test]
#[should_panic(expected = "assertion failed: points_to_something")]
fn varargs_o2() {
    let module = sound("varargs.c", 2);
    let _out = check(&module);
}
