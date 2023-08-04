use std::{collections::HashMap, path::Path, path::PathBuf, process::Command};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use yapall::{analysis::pointer, pointer::Options, Module, Operand, Signatures};

// ------------------------------------------------------------------
// Helpers (TODO: copied from tests)

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

// ------------------------------------------------------------------

fn module(program: &str, dir: &str, opt: u8) -> (Module, Signatures) {
    let llvm_module = compile(program, dir, opt);
    let mut operands: HashMap<Operand, &llvm_ir::Operand> = HashMap::new();
    let sigs = signatures(program, dir);
    match Module::new(&llvm_module, &mut operands) {
        Ok(m) => (m, sigs),
        Err(e) => panic!("{}", e),
    }
}

// ------------------------------------------------------------------

const OPTS: Options = Options {
    check_assertions: false,
    check_strict: false,
    contexts: 0,
    debug: false,
    metrics: false,
    unification: false,
};

pub fn any_o0(c: &mut Criterion) {
    let (m, sigs) = module("any.rs", "templates", 0);
    c.bench_function("pointer::analysis(any-O0)", |b| {
        b.iter(|| pointer::analysis(black_box(&m), &sigs, &OPTS))
    });
}

pub fn any_o1(c: &mut Criterion) {
    let (m, sigs) = module("any.rs", "templates", 1);
    c.bench_function("pointer::analysis(any-O1)", |b| {
        b.iter(|| pointer::analysis(black_box(&m), &sigs, &OPTS))
    });
}

pub fn cfg_o0(c: &mut Criterion) {
    let (m, sigs) = module("cfg-test.c", "property", 0);
    c.bench_function("pointer::analysis(cfg-O0)", |b| {
        b.iter(|| pointer::analysis(black_box(&m), &sigs, &OPTS))
    });
}

pub fn cfg_o1(c: &mut Criterion) {
    let (m, sigs) = module("cfg-test.c", "property", 1);
    c.bench_function("pointer::analysis(cfg-O1)", |b| {
        b.iter(|| pointer::analysis(black_box(&m), &sigs, &OPTS))
    });
}

pub fn linkedlist_o0(c: &mut Criterion) {
    let (m, sigs) = module("linkedlist.c", "property", 0);
    c.bench_function("pointer::analysis(linkedlist-O0)", |b| {
        b.iter(|| pointer::analysis(black_box(&m), &sigs, &OPTS))
    });
}

pub fn linkedlist_o1(c: &mut Criterion) {
    let (m, sigs) = module("linkedlist.c", "property", 1);
    c.bench_function("pointer::analysis(linkedlist-O1)", |b| {
        b.iter(|| pointer::analysis(black_box(&m), &sigs, &OPTS))
    });
}

pub fn vector_o0(c: &mut Criterion) {
    let (m, sigs) = module("vector.cpp", "templates", 0);
    c.bench_function("pointer::analysis(vector-O0)", |b| {
        b.iter(|| pointer::analysis(black_box(&m), &sigs, &OPTS))
    });
}

pub fn vector_o1(c: &mut Criterion) {
    let (m, sigs) = module("vector.cpp", "templates", 1);
    c.bench_function("pointer::analysis(vector-O1)", |b| {
        b.iter(|| pointer::analysis(black_box(&m), &sigs, &OPTS))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = any_o0, any_o1, cfg_o0, cfg_o1, linkedlist_o0, linkedlist_o1, vector_o0, vector_o1
}
criterion_main!(benches);
