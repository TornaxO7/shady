use image::{ImageBuffer, Rgba};
use pollster::FutureExt;
use shady::{Shady, ShadyDescriptor};
use wgpu::{
    Backends, Buffer, BufferView, Device, DeviceDescriptor, Extent3d, Instance, Queue,
    ShaderSource, Texture,
};
use winit::dpi::PhysicalSize;

use crate::states::{RenderState, SHADY_BIND_GROUP_INDEX, SHADY_VERTEX_BUFFER_INDEX};

type Bytes = u32;

/// https://www.w3.org/TR/webgpu/#gputexelcopybufferinfo
const MIN_BYTES_WIDTH: Bytes = 256;
const OUTPUT_BUFFER_VALUE_SIZE: u32 = std::mem::size_of::<u32>() as u32;

pub struct TextureState {
    size: PhysicalSize<u32>,
    texture: Texture,
    output_buffer: Buffer,
    texture_extent: Extent3d,

    device: Device,
    queue: Queue,
    shady: Shady,
}

impl TextureState {
    pub fn get_output(&self) -> ImageBuffer<Rgba<u8>, BufferView> {
        let buffer_slice = self.output_buffer.slice(..);

        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });

        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().expect("Retrieve output from buffer");

        let data = buffer_slice.get_mapped_range();

        ImageBuffer::from_raw(self.size.width, self.size.height, data)
            .expect("Create image buffer from wgpu output buffer")
    }

    pub fn new<'a>(
        texture_size: PhysicalSize<u32>,
        shader_source: Option<ShaderSource<'a>>,
    ) -> Self {
        assert!(
            MIN_BYTES_WIDTH / OUTPUT_BUFFER_VALUE_SIZE >= 64,
            "Width must be at least {}.",
            MIN_BYTES_WIDTH / OUTPUT_BUFFER_VALUE_SIZE
        );

        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .block_on()
            .expect("Create wgpu-adapter");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default(), None)
            .block_on()
            .expect("Retrieve device and queue");

        let output_buffer = {
            let buffer_size = (OUTPUT_BUFFER_VALUE_SIZE * texture_size.width * texture_size.height)
                as wgpu::BufferAddress;

            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Texture output buffer"),
                size: buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        };

        let texture_extent = wgpu::Extent3d {
            width: texture_size.width,
            height: texture_size.height,
            depth_or_array_layers: 1,
        };

        let texture_format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let shady = Shady::new(ShadyDescriptor {
            device: &device,
            initial_fragment_shader: shader_source,
            texture_format,
            bind_group_index: SHADY_BIND_GROUP_INDEX,
            vertex_buffer_index: SHADY_VERTEX_BUFFER_INDEX,
        });

        Self {
            size: texture_size,
            texture_extent,
            texture,
            device,
            queue,
            shady,
            output_buffer,
        }
    }
}

impl<'a> RenderState<'a> for TextureState {
    fn prepare_next_frame(&mut self) {
        self.shady.inc_frame();

        self.shady.update_audio_buffer(&mut self.queue);
        self.shady.update_frame_buffer(&mut self.queue);
        self.shady.update_mouse_buffer(&mut self.queue);
        self.shady.update_resolution_buffer(&mut self.queue);
        self.shady.update_time_buffer(&mut self.queue);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let texture_view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Shady texture command encoder"),
            });

        self.shady.add_render_pass(&mut encoder, &texture_view);

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                texture: &self.texture,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(std::mem::size_of::<u32>() as u32 * self.size.width),
                    rows_per_image: Some(self.size.height),
                },
            },
            self.texture_extent,
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    fn update_pipeline(&mut self, shader_source: ShaderSource<'a>) {
        self.shady.set_render_pipeline(&self.device, shader_source)
    }
}

#[test]
#[ignore] // nix build fails otherwise
fn red_screen() {
    let frag_code = "
            @fragment
            fn main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
                return vec4<f32>(1.0, 0.0, 0.0, 0.0);
            }
        ";

    let size = PhysicalSize {
        width: 64,
        height: 1,
    };

    let mut state = TextureState::new(
        size,
        Some(ShaderSource::Wgsl(std::borrow::Cow::Borrowed(frag_code))),
    );
    state.render().unwrap();

    let out = state.get_output();

    for pixel in out.pixels() {
        assert_eq!(pixel.0, [255, 0, 0, 0]);
    }
}
