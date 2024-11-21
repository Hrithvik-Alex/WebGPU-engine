use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

pub trait Component {
    fn name(&self) -> String;
}

pub struct VertexArrayComponent {
    pub vertices: Vec<cgmath::Vector2<f32>>,
    pub indices: Vec<u32>,
    pub tex_coords: Vec<cgmath::Vector2<f32>>,
}

impl Component for VertexArrayComponent {
    fn name(&self) -> String {
        return "VertexArray".to_string();
    }
}

impl VertexArrayComponent {
    pub fn quad() -> Self {
        let vertices = vec![
            cgmath::Vector2::new(0.0, 1.0), // TOP-LEFT
            cgmath::Vector2::new(1.0, 1.0), // TOP-RIGHT
            cgmath::Vector2::new(0.0, 0.0), // BOTTOM-LEFT
            cgmath::Vector2::new(1.0, 0.0), // BOTTOM-RIGHT
        ];

        let indices = vec![0, 2, 3, 0, 3, 1];

        let tex_coords = vec![
            cgmath::Vector2::new(0.0, 0.0), // TOP-LEFT
            cgmath::Vector2::new(1.0, 0.0), // TOP-RIGHT
            cgmath::Vector2::new(0.0, 1.0), // BOTTOM-LEFT
            cgmath::Vector2::new(1.0, 1.0), // BOTTOM-RIGHT
        ];

        Self {
            vertices,
            indices,
            tex_coords,
        }
    }
}

pub struct PositionComponent {
    pub position: cgmath::Vector2<f32>,
    pub scale: f32,
}

impl Component for PositionComponent {
    fn name(&self) -> String {
        return "Position".to_string();
    }
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WorldUniform {
    mat: [[f32; 4]; 4],
}

impl Component for WorldUniform {
    fn name(&self) -> String {
        "WorldUniform".to_string()
    }
}

impl WorldUniform {
    const WORLD_SCREEN_WIDTH: u32 = 640;
    const WORLD_SCREEN_HEIGHT: u32 = 360;

    pub fn new() -> Self {
        Self {
            mat: cgmath::Matrix4::identity().into(),
        }
    }

    fn calc(&self, width: u32, height: u32) -> cgmath::Matrix4<f32> {
        #[cfg_attr(rustfmt, rustfmt_skip)]
         cgmath::Matrix4::new(
            width as f32/Self::WORLD_SCREEN_WIDTH as f32, 0., 0., 0.,
            0., height as f32/Self::WORLD_SCREEN_HEIGHT as f32, 0., 0.,
            0., 0., 1., 0.,
            0., 0., 0., 1.,
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.mat = self.calc(width, height).into();
    }

    pub fn get_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ortho buffer"),
            contents: bytemuck::cast_slice(&[self.mat]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}
