use cgmath::SquareMatrix;
use slotmap::DenseSlotMap;
use wgpu::util::DeviceExt;

pub type Entity = slotmap::DefaultKey;
pub type EntityMap<T> = DenseSlotMap<Entity, Option<T>>;

pub trait Component {
    fn name(&self) -> String;
}
#[derive(Debug)]
pub struct VertexArrayComponent {
    pub vertices: Vec<cgmath::Vector2<f32>>,
    pub indices: Vec<u32>,
    pub whole_tex_coords: Vec<cgmath::Vector2<f32>>,
    pub tex_coords: Vec<cgmath::Vector2<f32>>,
    pub texture_index: u32,
    pub is_flipped: bool,
    pub z_value: f32,
}

impl Component for VertexArrayComponent {
    fn name(&self) -> String {
        return "VertexArray".to_string();
    }
}

impl VertexArrayComponent {
    pub const BACKGROUND_Z: f32 = 2.0;
    pub const FOREGROUND_Z: f32 = 1.0;
    pub const OBJECT_Z: f32 = 0.5;

    pub fn textured_quad(texture_index: u32, z_value: f32) -> Self {
        let vertices = vec![
            cgmath::Vector2::new(0.0, 1.0), // TOP-LEFT
            cgmath::Vector2::new(1.0, 1.0), // TOP-RIGHT
            cgmath::Vector2::new(0.0, 0.0), // BOTTOM-LEFT
            cgmath::Vector2::new(1.0, 0.0), // BOTTOM-RIGHT
        ];

        let indices = vec![0, 2, 3, 0, 3, 1];

        let whole_tex_coords = vec![
            cgmath::Vector2::new(0.0, 0.0), // TOP-LEFT
            cgmath::Vector2::new(1.0, 0.0), // TOP-RIGHT
            cgmath::Vector2::new(0.0, 1.0), // BOTTOM-LEFT
            cgmath::Vector2::new(1.0, 1.0), // BOTTOM-RIGHT
        ];

        Self {
            vertices,
            indices,
            tex_coords: whole_tex_coords.clone(),
            whole_tex_coords,
            texture_index,
            is_flipped: false,
            z_value,
        }
    }

    // pub fn sprite_quad(
    //     sprite_sheet: Arc<SpriteSheet>,
    //     sheet_position: cgmath::Vector2<u32>,
    // ) -> Self {
    //     let mut quad = Self::quad();
    //     sprite_sheet.adjust_tex_coords(&mut quad, sheet_position);
    //     quad
    // }
}

#[derive(Debug)]
pub struct PositionComponent {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub is_controllable: bool,
}

impl Component for PositionComponent {
    fn name(&self) -> String {
        return "Position".to_string();
    }
}

impl PositionComponent {
    pub fn scale_outward(&self, scale: cgmath::Vector2<f32>) -> Self {
        assert!(scale.x >= 1. && scale.y >= 1.);
        Self {
            position: self.position - scale / 2.,
            scale: self.scale + scale / 2.,
            is_controllable: self.is_controllable,
        }
    }
}

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

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum CharacterState {
    IDLE,
    MOVE,
    ATTACK,
}

#[derive(Debug)]
pub struct CharacterStateComponent {
    pub character_state: CharacterState,
}

impl Component for CharacterStateComponent {
    fn name(&self) -> String {
        return "Position".to_string();
    }
}
