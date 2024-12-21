pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;

    const ATTRIBS: [wgpu::VertexAttribute; 4];
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex2d {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal_coords: [f32; 2],

    pub extra_info: u32,
    // 00000000 00000000 000000f tttttttt
    // f -> is_flipped?
    // t -> 8 bit texture number
}

impl Vertex for ModelVertex2d {
    const ATTRIBS: [wgpu::VertexAttribute; 4] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x2, 3 => Uint32];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex2d>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
