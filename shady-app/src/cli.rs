use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    /// Path to the shaderfile.
    ///
    /// Must end with one of the following extensions:
    ///     - `.wgsl`
    pub fragment_path: PathBuf,

    /// Insert template to given shader.
    ///
    /// If enabled, the given shader will be prelpared for you so that you can immediately start writing your shader.
    #[arg(long)]
    pub template: bool,
}

pub fn parse() -> Args {
    Args::parse()
}
