use wgpu::util::DeviceExt;

type VertexCoord = [f32; 2];

const TOP_LEFT_CORNER: VertexCoord = [-1.0, 1.0];
const BOTTOM_LEFT_CORNER: VertexCoord = [-1.0, -1.0];
const BOTTOM_RIGHT_CORNER: VertexCoord = [1.0, -1.0];
const TOP_RIGHT_CORNER: VertexCoord = [1.0, 1.0];

const VERTICES: &[VertexCoord] = &[
    TOP_LEFT_CORNER,
    BOTTOM_LEFT_CORNER,
    BOTTOM_RIGHT_CORNER,
    TOP_RIGHT_CORNER,
];

#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    // left
    0, 1, 2,
    // right
    0, 2, 3,
];

pub const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<VertexCoord>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[wgpu::VertexAttribute {
        offset: 0 as wgpu::BufferAddress,
        shader_location: 0,
        format: wgpu::VertexFormat::Float32x2,
    }],
};

pub fn get_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Shady Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

pub fn get_index_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Shady Index Buffer"),
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    })
}
