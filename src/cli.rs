// SPDX-License-Identifier:i BSD-3-Clause
use std::fmt;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq, clap::ValueEnum)]
pub enum Check {
    Default,
    None,
    Strict,
}

impl std::fmt::Display for Check {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Check::Default => write!(f, "default"),
            Check::None => write!(f, "none"),
            Check::Strict => write!(f, "strict"),
        }
    }
}

/// Pointer analysis for LLVM bitcode
#[derive(Debug, clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Check assertions
    #[arg(long, default_value_t = Check::Default)]
    pub check: Check,

    /// Context depth
    #[arg(long, default_value_t = 0)]
    pub contexts: u8,

    /// Debug
    #[arg(long)]
    pub debug: bool,

    /// Quiet
    #[arg(long)]
    pub quiet: bool,

    /// Collect and report precision metrics
    #[arg(long)]
    pub metrics: bool,

    /// LLVM bitcode module
    #[arg()]
    pub module: PathBuf,

    /// Points-to signatures
    #[arg(short, long)]
    pub signatures: Option<PathBuf>,

    /// Tracing
    #[arg(long)]
    pub tracing: bool,

    /// Unification-based analysis
    #[arg(short, long)]
    pub unification: bool,
}
