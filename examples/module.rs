use std::path::PathBuf;

use clap::Parser;

/// Print LLVM bitcode module
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// LLVM bitcode module
    #[arg()]
    pub module: PathBuf,
}

fn main() {
    let args = Args::parse();
    let module = llvm_ir::Module::from_bc_path(args.module).unwrap();
    eprintln!("{:#?}", module.functions);
    eprintln!("{:#?}", module.global_vars);
}
