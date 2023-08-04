use std::{collections::HashMap, path::PathBuf, process::Command};

use yapall::{
    analysis::{callgraph, int},
    llvm::constant::Constant,
    llvm::instruction::{Call, Opcode},
    Arc, Callee, FunctionName, IntRelations, Invoke, Module, Operand, TerminatorOpcode, UArc,
};

// ------------------------------------------------------------------
// Helpers

fn compile(prog: &str, dir: &str, opt: u8) -> llvm_ir::Module {
    let out = format!("tests/{}/{}-{}.bc", dir, prog, opt);
    let compiler = if prog.ends_with(".c") {
        "clang"
    } else {
        "clang++"
    };
    let mut cmd = Command::new(compiler);
    cmd.arg("-emit-llvm")
        .arg("-c")
        .arg(format!("-O{}", opt))
        .arg("-Werror")
        .arg("-o")
        .arg(&out)
        .arg(format!("tests/{}/{}", dir, prog));
    let _status = cmd.output().unwrap();
    // assert!(status.success());
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
    let path = PathBuf::from(&out);
    llvm_ir::Module::from_bc_path(path).unwrap()
}

fn convert(program: &str, dir: &str, opt: u8) -> Module {
    let llvm_module = compile(program, dir, opt);
    let mut operands: HashMap<Arc<Operand>, &llvm_ir::Operand> = HashMap::new();
    match Module::new(&llvm_module, &mut operands) {
        Ok(m) => m,
        Err(e) => panic!("{}", e),
    }
}

fn check(module: &Module) -> IntRelations {
    let cg = callgraph::analysis(&module);
    let outs = int::analysis(&module, &cg, 2, false, false);
    check_module(&outs, module);
    outs
}

fn check_module(out: &IntRelations, module: &Module) {
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

fn check_call(
    out: &IntRelations,
    _caller: &UArc<FunctionName>,
    callee: &Callee,
    arguments: &Vec<Arc<Operand>>,
) {
    if let Callee::Operand(op) = &callee {
        if let Operand::Constant(const_arc) = &**op {
            if let Constant::Function(name) = &**const_arc {
                if **name == FunctionName::from("assert_constant") {
                    for arg in arguments {
                        let mut found = false;
                        for ((_, op), val) in &out.operand_val {
                            if op == arg {
                                assert!(*val != yapall::IntLattice::top());
                                found = true;
                                break;
                            }
                        }
                        assert!(found);
                    }
                } else if (**name).to_string().starts_with("@assert_") {
                    panic!("Unknown assertion: {name}")
                }
            }
        }
    }
}
// ------------------------------------------------------------------

#[test]
#[ignore]
fn int_properties() {
    for program in std::fs::read_dir("tests/property").unwrap() {
        let path = program.as_ref().unwrap().path();
        println!("{}", path.to_string_lossy());
        if path.to_string_lossy().ends_with(".c") {
            // || path.to_string_lossy().ends_with(".cpp") {
            let module = convert(
                path.strip_prefix("tests/property")
                    .unwrap()
                    .to_str()
                    .unwrap(),
                "property",
                1,
            );
            let _out = check(&module);
        }
    }
}

#[test]
fn int_arg() {
    let module = convert("arg.c", "int", 1);
    let _out = check(&module);
}

#[test]
fn int_const() {
    let module = convert("const.c", "int", 1);
    let _out = check(&module);
}

#[test]
fn int_ret() {
    let module = convert("ret.c", "int", 1);
    let _out = check(&module);
}
