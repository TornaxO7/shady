#[cfg(feature = "audio")]
mod audio;
#[cfg(feature = "frame")]
mod frame;
#[cfg(feature = "mouse")]
mod mouse;
#[cfg(feature = "resolution")]
mod resolution;
#[cfg(feature = "time")]
mod time;

use std::fmt;

#[cfg(feature = "audio")]
use audio::Audio;
#[cfg(feature = "frame")]
use frame::Frame;
#[cfg(feature = "mouse")]
use mouse::Mouse;
#[cfg(feature = "resolution")]
use resolution::Resolution;
#[cfg(feature = "time")]
use time::Time;

use tracing::instrument;
use wgpu::Device;

#[cfg(feature = "mouse")]
pub use mouse::MouseState;

use crate::{template::TemplateGenerator, ShadyDescriptor};

#[repr(u32)]
enum BindingValue {
    #[cfg(feature = "audio")]
    Audio,
    #[cfg(feature = "frame")]
    Frame,
    #[cfg(feature = "mouse")]
    Mouse,
    #[cfg(feature = "resolution")]
    Resolution,
    #[cfg(feature = "time")]
    Time,
}

pub trait Resource: TemplateGenerator {
    fn new(desc: &ShadyDescriptor) -> Self;

    fn binding() -> u32;

    fn buffer_label() -> &'static str;

    fn update_buffer(&self, queue: &wgpu::Queue);

    fn buffer(&self) -> &wgpu::Buffer;

    fn buffer_type() -> wgpu::BufferBindingType;

    // `unused`: For example if the dev enables just the `audio` feature, this function wouldn't be used.
    #[allow(unused)]
    fn create_uniform_buffer(device: &Device, size: u64) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(Self::buffer_label()),
            size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    // `unused`: For example if the dev enables just the `time` feature, this function wouldn't be used.
    #[allow(unused)]
    fn create_storage_buffer(device: &Device, size: u64) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(Self::buffer_label()),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}

pub struct Resources {
    #[cfg(feature = "audio")]
    pub audio: Audio,
    #[cfg(feature = "frame")]
    pub frame: Frame,
    #[cfg(feature = "mouse")]
    pub mouse: Mouse,
    #[cfg(feature = "resolution")]
    pub resolution: Resolution,
    #[cfg(feature = "time")]
    pub time: Time,
}

impl Resources {
    #[instrument(level = "trace", skip_all)]
    pub fn new(desc: &ShadyDescriptor) -> Self {
        Self {
            #[cfg(feature = "audio")]
            audio: Audio::new(desc),
            #[cfg(feature = "frame")]
            frame: Frame::new(desc),
            #[cfg(feature = "mouse")]
            mouse: Mouse::new(desc),
            #[cfg(feature = "resolution")]
            resolution: Resolution::new(desc),
            #[cfg(feature = "time")]
            time: Time::new(desc),
        }
    }
}

/// Methods regarding bind groups
impl Resources {
    #[instrument(level = "trace")]
    pub fn bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shady bind group layout"),
            entries: &[
                #[cfg(feature = "audio")]
                bind_group_layout_entry(Audio::binding(), Audio::buffer_type()),
                #[cfg(feature = "frame")]
                bind_group_layout_entry(Frame::binding(), Frame::buffer_type()),
                #[cfg(feature = "mouse")]
                bind_group_layout_entry(Mouse::binding(), Mouse::buffer_type()),
                #[cfg(feature = "resolution")]
                bind_group_layout_entry(Resolution::binding(), Resolution::buffer_type()),
                #[cfg(feature = "time")]
                bind_group_layout_entry(Time::binding(), Time::buffer_type()),
            ],
        })
    }

    #[instrument(skip(self), level = "trace")]
    pub fn bind_group(&self, device: &Device) -> wgpu::BindGroup {
        let layout = Self::bind_group_layout(device);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shady bind group"),
            layout: &layout,
            entries: &[
                #[cfg(feature = "audio")]
                wgpu::BindGroupEntry {
                    binding: Audio::binding(),
                    resource: self.audio.buffer().as_entire_binding(),
                },
                #[cfg(feature = "frame")]
                wgpu::BindGroupEntry {
                    binding: Frame::binding(),
                    resource: self.frame.buffer().as_entire_binding(),
                },
                #[cfg(feature = "mouse")]
                wgpu::BindGroupEntry {
                    binding: Mouse::binding(),
                    resource: self.mouse.buffer().as_entire_binding(),
                },
                #[cfg(feature = "resolution")]
                wgpu::BindGroupEntry {
                    binding: Resolution::binding(),
                    resource: self.resolution.buffer().as_entire_binding(),
                },
                #[cfg(feature = "time")]
                wgpu::BindGroupEntry {
                    binding: Time::binding(),
                    resource: self.time.buffer().as_entire_binding(),
                },
            ],
        })
    }
}

impl TemplateGenerator for Resources {
    fn write_wgsl_template(
        writer: &mut dyn fmt::Write,
        bind_group_index: u32,
    ) -> Result<(), fmt::Error> {
        #[cfg(feature = "audio")]
        Audio::write_wgsl_template(writer, bind_group_index)?;
        #[cfg(feature = "frame")]
        Frame::write_wgsl_template(writer, bind_group_index)?;
        #[cfg(feature = "mouse")]
        Mouse::write_wgsl_template(writer, bind_group_index)?;
        #[cfg(feature = "resolution")]
        Resolution::write_wgsl_template(writer, bind_group_index)?;
        #[cfg(feature = "time")]
        Time::write_wgsl_template(writer, bind_group_index)?;

        Ok(())
    }

    fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
        #[cfg(feature = "audio")]
        Audio::write_glsl_template(writer)?;
        #[cfg(feature = "frame")]
        Frame::write_glsl_template(writer)?;
        #[cfg(feature = "mouse")]
        Mouse::write_glsl_template(writer)?;
        #[cfg(feature = "resolution")]
        Resolution::write_glsl_template(writer)?;
        #[cfg(feature = "time")]
        Time::write_glsl_template(writer)?;

        Ok(())
    }
}

#[instrument(level = "trace")]
fn bind_group_layout_entry(
    binding: u32,
    ty: wgpu::BufferBindingType,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}
