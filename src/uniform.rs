use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

use crate::component::Component;

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WorldUniform {
    mat: [[f32; 4]; 4],
}

// impl Component for WorldUniform {
//     fn name(&self) -> String {
//         "WorldUniform".to_string()
//     }
// }

impl WorldUniform {
    pub const WORLD_SCREEN_WIDTH: u32 = 640;
    pub const WORLD_SCREEN_HEIGHT: u32 = 360;

    pub fn new() -> Self {
        Self {
            mat: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn calc(&self, width: u32, height: u32) -> cgmath::Matrix4<f32> {
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

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TimeUniform {
    pub time: f32,
}

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    pub intensity: f32,
    pub color: [f32; 3],
    pub padding: f32,
}

pub struct LightComponent {
    pub intensity: f32,
    pub color: cgmath::Vector3<f32>,
}

impl Component for LightComponent {
    fn name(&self) -> String {
        return "Light".to_string();
    }
}
