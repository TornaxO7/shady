use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    /// Path to the shaderfile.
    ///
    /// Must end with one of the following extensions:
    ///
    ///     - `.wgsl`
    ///
    ///     - `.glsl`
    ///
    /// Shady-App will automatically detect which shader-syntax it should use, depending on the extension.
    ///
    /// So for example, if you use `/dir1/dir2/fragment_shader.glsl` Shady-App will treat the given file
    /// as a `glsl` shader.
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
