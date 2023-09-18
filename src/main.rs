// SPDX-License-Identifier: BSD-3-Clause
use std::collections::HashMap;
use std::io::{self, Write};

use anyhow::{anyhow, Context, Error, Result};
use clap::Parser;

use tracing_flame::FlameLayer;
use tracing_subscriber::{fmt, prelude::*};

mod alloc;
pub mod analysis;
mod arc;
mod cli;
mod hash;
mod klimited;
mod lattice;
mod layers;
mod llvm;
mod signatures;
mod union;

use analysis::pointer;

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
        .with(layers::NanoCountLayer::default())
        .init();
    _guard
}

fn main() -> Result<()> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let args = cli::Args::parse();

    if args.tracing {
        setup_global_subscriber();
    }

    let signatures = if let Some(signatures_path) = args.signatures {
        let signatures_string = std::fs::read_to_string(signatures_path)
            .context("Couldn't read points-to signatures")?;
        signatures::Signatures::new(
            serde_json::from_str(&signatures_string)
                .context("Couldn't deserialize points-to signatures")?,
        )
        .context("Couldn't construct points-to signatures")?
    } else {
        signatures::Signatures::default()
    };

    let llvm_module = llvm_ir::Module::from_bc_path(&args.module)
        .map_err(Error::msg)
        .with_context(|| {
            format!(
                "Couldn't parse LLVM bitcode module at {}",
                args.module.display()
            )
        })?;
    let mut operands: HashMap<crate::arc::Arc<llvm::Operand>, &llvm_ir::Operand> =
    // just a guess:
        HashMap::with_capacity(llvm_module.global_vars.len() + (8 * llvm_module.functions.len()));
    let module = llvm::Module::new(&llvm_module, &mut operands).context("Malformed LLVM module")?;
    drop(operands);

    let opts = pointer::Options {
        check_assertions: args.check == cli::Check::Default || args.check == cli::Check::Strict,
        check_strict: args.check == cli::Check::Strict,
        contexts: args.contexts,
        debug: args.debug,
        metrics: args.metrics,
        unification: args.unification,
    };
    let outs = pointer::analysis(&module, &signatures, &opts);

    if !args.quiet {
        let mut stdout = io::stdout().lock();
        writeln!(stdout, "reachable")?;
        writeln!(stdout, "---------")?;
        for f in &outs.reachable {
            writeln!(stdout, "{}", *f)?;
        }
        writeln!(stdout)?;
        writeln!(stdout, "operand_points_to")?;
        writeln!(stdout, "-----------------")?;
        // TODO: Improve printing of contexts
        for pts in &outs.operand_points_to {
            writeln!(
                stdout,
                "{:?}: {} --> {}",
                (*pts.0).clone().into_iter().collect::<Vec<_>>(),
                *pts.1,
                *pts.2
            )?;
        }
        writeln!(stdout)?;
        writeln!(stdout, "alloc_points_to")?;
        writeln!(stdout, "---------------")?;
        for pts in &outs.alloc_points_to {
            writeln!(stdout, "{} --> {}", *pts.0, *pts.1)?;
        }
        writeln!(stdout)?;
        writeln!(stdout, "needs_signature")?;
        writeln!(stdout, "---------------")?;
        for f in &outs.needs_signature {
            writeln!(stdout, "{}", *f)?;
        }
    }

    if args.metrics {
        let mut stdout = io::stdout().lock();
        if let Some(m) = outs.metrics {
            writeln!(stdout)?;
            writeln!(stdout, "metrics")?;
            writeln!(stdout, "-------")?;
            writeln!(stdout, "callgraph size: {}", m.callgraph_size)?;
            writeln!(stdout, "free of non-heap allocation: {}", m.free_non_heap)?;
            writeln!(stdout, "invalid calls: {}", m.invalid_calls)?;
            writeln!(stdout, "invalid loads: {}", m.invalid_loads)?;
            writeln!(stdout, "invalid memcpy dsts: {}", m.invalid_memcpy_dsts)?;
            writeln!(stdout, "invalid memcpy srcs: {}", m.invalid_memcpy_srcs)?;
            writeln!(stdout, "invalid stores: {}", m.invalid_stores)?;
            writeln!(stdout, "points-to top: {}", m.points_to_top)?;
        }
    }

    if let cli::Check::Strict = args.check {
        if !outs.needs_signature.is_empty() {
            return Err(anyhow!("Found functions that need signatures!"));
        }
    }

    Ok(())
}
