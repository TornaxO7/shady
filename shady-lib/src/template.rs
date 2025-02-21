//! Module to generate templates which can be modified and used for [Shady].
use std::fmt;

use crate::{resources::Resources, BIND_GROUP_INDEX, FRAGMENT_ENTRYPOINT};

/// The shader languages where the templates can be generated for.
///
/// # Example
/// ```
/// use shady::TemplateLang;
///
/// // Create a template in wgsl
/// let template = TemplateLang::Wgsl
///     .generate_to_string()
///     .unwrap();
/// ```
#[derive(Debug, Clone, Copy, Hash)]
pub enum TemplateLang {
    /// A template in the wgsl shader language.
    Wgsl,

    /// The glsl shader language.
    Glsl,
}

pub(crate) trait TemplateGenerator {
    fn write_wgsl_template(
        writer: &mut dyn fmt::Write,
        bind_group_index: u32,
    ) -> Result<(), fmt::Error>;

    fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error>;
}

impl TemplateLang {
    /// Create the template and return them as a String.
    ///
    /// # Example
    /// ```
    /// use shady::TemplateLang;
    ///
    /// // Create a template in wgsl
    /// let template = TemplateLang::Wgsl
    ///     .generate_to_string()
    ///     .unwrap();
    /// ```
    pub fn generate_to_string(self) -> Result<String, fmt::Error> {
        let mut string = String::new();
        self.generate(&mut string)?;
        Ok(string)
    }

    /// Create the template and write it to the given `writer`.
    ///
    /// # Example
    /// ```
    /// use shady::TemplateLang;
    ///
    /// let mut template = String::new();
    ///
    /// // Generate the template and store it into `template`.
    /// TemplateLang::Wgsl
    ///     .generate(&mut template)
    ///     .unwrap();
    /// ```
    pub fn generate(self, writer: &mut dyn std::fmt::Write) -> Result<(), fmt::Error> {
        match self {
            TemplateLang::Wgsl => {
                Resources::write_wgsl_template(writer, BIND_GROUP_INDEX)?;

                writer.write_fmt(format_args!(
                    "
@fragment
fn {}(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {{
    let uv = pos.xy/iResolution.xy;
    let col = 0.5 + 0.5 * cos(iTime + uv.xyx + vec3<f32>(0.0, 2.0, 4.0));

    return vec4<f32>(col, 1.0);
}}
",
                    FRAGMENT_ENTRYPOINT
                ))?;
            }

            TemplateLang::Glsl => {
                Resources::write_glsl_template(writer)?;

                writer.write_fmt(format_args!(
                    "
// the color which the pixel should have
layout(location = 0) out vec4 fragColor;

void {}() {{
    // Normalized pixel coordinates (from 0 to 1)
    vec2 uv = gl_FragCoord.xy/iResolution.xy;

    // Time varying pixel color
    vec3 col = 0.5 + 0.5*cos(iTime+uv.xyx+vec3(0,2,4));

    // Output to screen
    fragColor = vec4(col,1.0);      
}}
",
                    FRAGMENT_ENTRYPOINT
                ))?;
            }
        };

        Ok(())
    }
}
