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

pub use mouse::MouseState;

pub trait Resource {
    type BufferDataType;

    fn new(device: &Device, binding: u32) -> Self;

    fn binding(&self) -> u32;

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
        const INIT_BINDING: u32 = 0;

        Self {
            time: Time::new(device, INIT_BINDING),
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
}

/// Methods regardin bind groups
impl Resources {
    #[instrument(skip(self), level = "trace")]
    pub fn bind_group_layout(&self, device: &Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Shady bind group layout"),
            entries: &[
                bind_group_layout_uniform_entry(self.time.binding(), Time::buffer_type()),
                bind_group_layout_uniform_entry(
                    self.resolution.binding(),
                    Resolution::buffer_type(),
                ),
                #[cfg(feature = "audio")]
                bind_group_layout_uniform_entry(self.audio.binding(), Audio::buffer_type()),
                bind_group_layout_uniform_entry(self.frame.binding(), Frame::buffer_type()),
                bind_group_layout_uniform_entry(self.mouse.binding(), Mouse::buffer_type()),
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
fn bind_group_layout_uniform_entry(
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
