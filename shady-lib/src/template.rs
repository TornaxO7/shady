use std::fmt;

use crate::{resources::Resources, FRAGMENT_ENTRYPOINT};

#[derive(Debug, Clone, Copy, Hash)]
pub enum TemplateLang {
    Wgsl { bind_group_index: u32 },
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
    pub fn generate_to_string(self) -> Result<String, fmt::Error> {
        let mut string = String::new();
        self.generate(&mut string)?;
        Ok(string)
    }

    pub fn generate(self, writer: &mut dyn std::fmt::Write) -> Result<(), fmt::Error> {
        match self {
            TemplateLang::Wgsl { bind_group_index } => {
                Resources::write_wgsl_template(writer, bind_group_index)?;

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
