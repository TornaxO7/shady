#[cfg(feature = "audio")]
mod audio;
mod frame;
mod mouse;
mod resolution;
mod time;

#[cfg(feature = "audio")]
use audio::Audio;
use frame::Frame;
use mouse::Mouse;
use resolution::Resolution;
use time::Time;
use tracing::instrument;
use wgpu::Device;

pub trait Uniform {
    type BufferDataType;

    fn new(device: &Device, binding: u32) -> Self;

    fn binding(&self) -> u32;

    fn buffer_label() -> &'static str;

    fn update_buffer(&self, queue: &mut wgpu::Queue);

    fn cleanup(&mut self);

    fn buffer(&self) -> &wgpu::Buffer;

    fn create_buffer(device: &Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&Self::buffer_label()),
            size: std::mem::size_of::<Self::BufferDataType>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}

pub struct Uniforms {
    pub time: Time,
    pub resolution: Resolution,
    #[cfg(feature = "audio")]
    pub audio: Audio,
    pub frame: Frame,
    pub mouse: Mouse,
}

impl Uniforms {
    #[instrument(level = "trace")]
    pub fn new(device: &wgpu::Device) -> Self {
        const INIT_BINDING: u32 = 0;

        Self {
            time: Time::new(device, INIT_BINDING + 0),
            resolution: Resolution::new(device, INIT_BINDING + 1),
            #[cfg(feature = "audio")]
            audio: Audio::new(device, INIT_BINDING + 2),
            mouse: Mouse::new(device, INIT_BINDING + 3),
            frame: Frame::new(device, INIT_BINDING + 4),
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

    #[instrument(skip_all, level = "trace")]
    pub fn cleanup(&mut self) {
        self.time.cleanup();
        self.resolution.cleanup();
        #[cfg(feature = "audio")]
        self.audio.cleanup();
        self.frame.cleanup();
        self.mouse.cleanup();
    }
}

/// Methods regardin bind groups
impl Uniforms {
    #[instrument(skip(self), level = "trace")]
    pub fn bind_group_layout(&self, device: &Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shady bind group layout"),
            entries: &[
                bind_group_layout_entry(self.time.binding()),
                bind_group_layout_entry(self.resolution.binding()),
                #[cfg(feature = "audio")]
                bind_group_layout_entry(self.audio.binding()),
                bind_group_layout_entry(self.frame.binding()),
                bind_group_layout_entry(self.mouse.binding()),
            ],
        })
    }

    #[instrument(skip(self), level = "trace")]
    pub fn bind_group(&self, device: &Device) -> wgpu::BindGroup {
        let layout = self.bind_group_layout(device);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shady bind group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: self.time.binding(),
                    resource: self.time.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: self.resolution.binding(),
                    resource: self.resolution.buffer().as_entire_binding(),
                },
                #[cfg(feature = "audio")]
                wgpu::BindGroupEntry {
                    binding: self.audio.binding(),
                    resource: self.audio.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: self.frame.binding(),
                    resource: self.frame.buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: self.mouse.binding(),
                    resource: self.mouse.buffer().as_entire_binding(),
                },
            ],
        })
    }
}

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
