use std::fmt;

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
