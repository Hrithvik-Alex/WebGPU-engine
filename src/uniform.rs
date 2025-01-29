use cgmath::SquareMatrix;
use log::debug;
use wgpu::util::DeviceExt;

use crate::component::Component;

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WorldUniform {
    pub world_to_screen: [[f32; 4]; 4],
    pub screen_to_world: [[f32; 4]; 4],
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
            world_to_screen: cgmath::Matrix4::identity().into(),
            screen_to_world: cgmath::Matrix4::identity().into(),
        }
    }

    fn calc(width: u32, height: u32) -> cgmath::Matrix4<f32> {
        #[cfg_attr(rustfmt, rustfmt_skip)]
         cgmath::Matrix4::new(
            width as f32/Self::WORLD_SCREEN_WIDTH as f32, 0., 0., 0.,
            0., -1. * height as f32/Self::WORLD_SCREEN_HEIGHT as f32, 0., 0.,
            0., 0., 1., 0.,
            0., height as f32, 0., 1.,
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        let mat = Self::calc(width, height);
        if !(mat * 1000.).is_invertible() {
            debug!("{:?}", mat);
            assert!(false); // I want to know if this ever happens... lol
        }
        self.world_to_screen = mat.into();
        self.screen_to_world = mat.invert().unwrap().into();
    }

    pub fn get_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("world uniform"),
            contents: bytemuck::cast_slice(&[self.world_to_screen, self.screen_to_world]),
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
    pub linear_dropoff: f32,
    pub color: [f32; 3],
    pub quadratic_dropoff: f32,
    pub ambient_strength: f32,
    pub diffuse_strength: f32,
    pub padding: [f32; 2],
}

pub struct LightComponent {
    pub color: cgmath::Vector3<f32>,
    pub linear_dropoff: f32,
    pub quadratic_dropoff: f32,
    pub ambient_strength: f32,
    pub diffuse_strength: f32,
}

impl Component for LightComponent {
    fn name(&self) -> String {
        return "Light".to_string();
    }
}
