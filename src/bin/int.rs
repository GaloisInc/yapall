// SPDX-License-Identifier:i BSD-3-Clause
use std::collections::HashMap;
use std::io::{self, Write};

use anyhow::{Context, Error, Result};
use clap::Parser;

use tracing_flame::FlameLayer;
use tracing_subscriber::{fmt, prelude::*};

use yapall::llvm;

/// Pointer analysis for LLVM bitcode
#[derive(Debug, clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Context depth
    #[arg(long, default_value_t = 0)]
    pub contexts: u8,

    /// Debug
    #[arg(long)]
    pub debug: bool,

    /// Quiet
    #[arg(long)]
    pub quiet: bool,

    /// LLVM bitcode module
    #[arg()]
    pub module: std::path::PathBuf,

    #[arg(long)]
    pub metrics: bool,

    /// Tracing
    #[arg(long)]
    pub tracing: bool,
}

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn setup_global_subscriber() -> impl Drop {
    let filter_layer = tracing::level_filters::LevelFilter::TRACE;
    let fmt_layer = fmt::Layer::default();
    // TODO: Flamegraph doesn't seem to be working...
    let (flame_layer, _guard) = FlameLayer::with_file("./tracing.folded").unwrap();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(flame_layer)
        .with(yapall::layers::NanoCountLayer::default())
        .init();
    _guard
}

fn main() -> Result<()> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let args = Args::parse();

    if args.tracing {
        setup_global_subscriber();
    }

    let llvm_module = llvm_ir::Module::from_bc_path(&args.module)
        .map_err(Error::msg)
        .with_context(|| {
            format!(
                "Couldn't parse LLVM bitcode module at {}",
                args.module.display()
            )
        })?;
    let mut operands: HashMap<yapall::arc::Arc<llvm::Operand>, &llvm_ir::Operand> =
    // just a guess:
        HashMap::with_capacity(llvm_module.global_vars.len() + (8 * llvm_module.functions.len()));
    let module = llvm::Module::new(&llvm_module, &mut operands).context("Malformed LLVM module")?;
    drop(operands);

    let cg = yapall::analysis::callgraph::analysis(&module);
    let outs = yapall::analysis::int::analysis(
        &module,
        &cg,
        args.contexts.into(),
        args.debug,
        args.metrics,
    );

    if !args.quiet {
        let mut stdout = io::stdout().lock();
        for ((ctx, op), val) in outs.operand_val {
            writeln!(
                stdout,
                "{:?} ⊢ {} = {}",
                (*ctx).clone().into_iter().collect::<Vec<_>>(),
                op,
                val
            )?;
        }
    }

    if args.metrics {
        let mut stdout = io::stdout().lock();
        if let Some(m) = outs.metrics {
            writeln!(stdout)?;
            writeln!(stdout, "metrics")?;
            writeln!(stdout, "-------")?;
            writeln!(stdout, "tops: {}", m.tops)?;
        }
    }
    Ok(())
}
