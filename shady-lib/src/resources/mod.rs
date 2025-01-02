#[cfg(feature = "audio")]
mod audio;
mod frame;
mod mouse;
mod resolution;
mod time;

use std::fmt;

#[cfg(feature = "audio")]
use audio::Audio;
use frame::Frame;
use mouse::Mouse;
use resolution::Resolution;
use time::Time;
use tracing::instrument;
use wgpu::Device;

pub use mouse::MouseState;

use crate::template::TemplateGenerator;

#[repr(u32)]
enum BindingValue {
    Time,
    Resolution,
    Audio,
    Mouse,
    Frame,
}

pub trait Resource: TemplateGenerator {
    type BufferDataType;

    fn new(device: &Device) -> Self;

    fn binding() -> u32;

    fn buffer_label() -> &'static str;

    fn update_buffer(&self, queue: &mut wgpu::Queue);

    fn buffer(&self) -> &wgpu::Buffer;

    fn buffer_type() -> wgpu::BufferBindingType;

    fn create_uniform_buffer(device: &Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(Self::buffer_label()),
            size: std::mem::size_of::<Self::BufferDataType>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_storage_buffer(device: &Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&Self::buffer_label()),
            size: std::mem::size_of::<Self::BufferDataType>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}

pub struct Resources {
    pub time: Time,
    pub resolution: Resolution,
    #[cfg(feature = "audio")]
    pub audio: Audio,
    pub frame: Frame,
    pub mouse: Mouse,
}

impl Resources {
    #[instrument(level = "trace")]
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            time: Time::new(device),
            resolution: Resolution::new(device),
            #[cfg(feature = "audio")]
            audio: Audio::new(device),
            mouse: Mouse::new(device),
            frame: Frame::new(device),
        }
    }

    #[instrument(skip_all, level = "trace")]
    pub fn update_buffers(&mut self, queue: &mut wgpu::Queue) {
        self.time.update_buffer(queue);
        self.resolution.update_buffer(queue);

        #[cfg(feature = "audio")]
        self.audio.update_buffer(queue);
        self.frame.update_buffer(queue);
        self.mouse.update_buffer(queue);
    }
}

/// Methods regarding bind groups
impl Resources {
    #[instrument(level = "trace")]
    pub fn bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shady bind group layout"),
            entries: &[
                bind_group_layout_entry(Time::binding(), Time::buffer_type()),
                bind_group_layout_entry(Resolution::binding(), Resolution::buffer_type()),
                #[cfg(feature = "audio")]
                bind_group_layout_entry(Audio::binding(), Audio::buffer_type()),
                bind_group_layout_entry(Frame::binding(), Frame::buffer_type()),
                bind_group_layout_entry(Mouse::binding(), Mouse::buffer_type()),
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
                wgpu::BindGroupEntry {
                    binding: Time::binding(),
                    resource: self.time.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Resolution::binding(),
                    resource: self.resolution.buffer().as_entire_binding(),
                },
                #[cfg(feature = "audio")]
                wgpu::BindGroupEntry {
                    binding: Audio::binding(),
                    resource: self.audio.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Frame::binding(),
                    resource: self.frame.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Mouse::binding(),
                    resource: self.mouse.buffer().as_entire_binding(),
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
        Time::write_wgsl_template(writer, bind_group_index)?;
        Resolution::write_wgsl_template(writer, bind_group_index)?;
        Audio::write_wgsl_template(writer, bind_group_index)?;
        Mouse::write_wgsl_template(writer, bind_group_index)?;
        Frame::write_wgsl_template(writer, bind_group_index)?;

        Ok(())
    }

    fn write_glsl_template(writer: &mut dyn fmt::Write) -> Result<(), fmt::Error> {
        Time::write_glsl_template(writer)?;
        Resolution::write_glsl_template(writer)?;
        Audio::write_glsl_template(writer)?;
        Mouse::write_glsl_template(writer)?;
        Frame::write_glsl_template(writer)?;

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
