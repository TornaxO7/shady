use std::path::Path;

const WGSL_EXTENSION: &str = "wgsl";
const GLSL_EXTENSION: &str = "glsl";

#[derive(Debug, Clone, Copy)]
pub enum ShaderLanguage {
    Wgsl,
    Glsl,
}

impl TryFrom<&Path> for ShaderLanguage {
    type Error = String;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let extension = path
            .extension()
            .ok_or("No extension".to_string())?
            .to_str()
            .unwrap();

        match extension {
            WGSL_EXTENSION => Ok(Self::Wgsl),
            GLSL_EXTENSION => Ok(Self::Glsl),
            other => Err(format!(
                "Unknown file extension to determine shader language: {}",
                other
            )),
        }
    }
}
