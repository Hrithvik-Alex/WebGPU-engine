pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;

    const ATTRIBS: [wgpu::VertexAttribute; 5];
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex2d {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
    pub normal_coords: [f32; 2],
    pub texture: u32,
    pub padding: f32,
}

impl Vertex for ModelVertex2d {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x2, 3 => Uint32, 4 => Float32];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex2d>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
