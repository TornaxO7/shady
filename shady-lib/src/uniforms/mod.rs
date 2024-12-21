mod audio;
mod resolution;
mod time;

use audio::Audio;
use resolution::Resolution;
use time::Time;
use tracing::instrument;
use wgpu::Device;

pub trait Uniform {
    type BufferDataType;

    fn binding() -> u32;

    fn buffer_label() -> &'static str;

    fn update_buffer(&self, queue: &mut wgpu::Queue, device: &wgpu::Device);

    fn cleanup(&mut self);

    fn buffer(device: &Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(Self::buffer_label()),
            size: std::mem::size_of::<Self::BufferDataType>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}

pub struct Uniforms {
    pub time: Time,
    pub resolution: Resolution,
    pub audio: Audio,
}

impl Uniforms {
    #[instrument(level = "trace")]
    pub fn new() -> Self {
        Self {
            time: Time::default(),
            resolution: Resolution::default(),
            audio: Audio::new(),
        }
    }

    #[instrument(skip_all, level = "trace")]
    pub fn update_buffers(&self, queue: &mut wgpu::Queue, device: &wgpu::Device) {
        self.time.update_buffer(queue, device);
        self.resolution.update_buffer(queue, device);
        self.audio.update_buffer(queue, device);
    }

    #[instrument(skip_all, level = "trace")]
    pub fn cleanup(&mut self) {
        self.time.cleanup();
        self.resolution.cleanup();
        self.audio.cleanup();
    }
}

/// Methods regarding bind groups
#[instrument(level = "trace")]
fn bind_group_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

#[instrument(level = "trace")]
pub fn bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Shady bind group layout"),
        entries: &[
            bind_group_layout_entry(Time::binding()),
            bind_group_layout_entry(Resolution::binding()),
            bind_group_layout_entry(Audio::binding()),
        ],
    })
}

#[instrument(level = "trace")]
pub fn bind_group(device: &Device) -> wgpu::BindGroup {
    let layout = bind_group_layout(device);

    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Shady bind group"),
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: Time::binding(),
                resource: Time::buffer(device).as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: Resolution::binding(),
                resource: Resolution::buffer(device).as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: Audio::binding(),
                resource: Audio::buffer(device).as_entire_binding(),
            },
        ],
    })
}
