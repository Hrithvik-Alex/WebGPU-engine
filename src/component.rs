use std::f64;

use cgmath::ElementWise;
use slotmap::DenseSlotMap;

use crate::physics;

pub type Entity = slotmap::DefaultKey;
pub type EntityMap<T> = DenseSlotMap<Entity, Option<T>>;

pub trait Component {
    fn name(&self) -> String;
}

#[derive(Debug, PartialEq)]
pub enum ShaderType {
    STANDARD,
    COLLECTIBLE,
}

#[derive(Debug)]
pub struct VertexArrayComponent {
    pub vertices: Vec<cgmath::Vector2<f32>>,
    pub indices: Vec<u32>,
    pub whole_tex_coords: Vec<cgmath::Vector2<f32>>,
    pub tex_coords: Vec<cgmath::Vector2<f32>>,
    pub texture_index: u32,
    pub is_flipped: bool,
    pub shader_type: ShaderType,
    // TODO: this should maybe be in positioncomponent
    pub z_value: f32,
    //TODO: hacky
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

    pub fn textured_quad_with_coords(
        texture_index: u32,
        z_value: f32,
        tex_coord_scale: cgmath::Vector2<f32>,
    ) -> Self {
        let vertices = vec![
            cgmath::Vector2::new(-0.5, 0.5),  // TOP-LEFT
            cgmath::Vector2::new(0.5, 0.5),   // TOP-RIGHT
            cgmath::Vector2::new(-0.5, -0.5), // BOTTOM-LEFT
            cgmath::Vector2::new(0.5, -0.5),  // BOTTOM-RIGHT
        ];

        let indices = vec![0, 2, 3, 0, 3, 1];

        let whole_tex_coords: Vec<cgmath::Vector2<f32>> = vec![
            cgmath::Vector2::new(0.0, 0.0), // TOP-LEFT
            cgmath::Vector2::new(1.0, 0.0), // TOP-RIGHT
            cgmath::Vector2::new(0.0, 1.0), // BOTTOM-LEFT
            cgmath::Vector2::new(1.0, 1.0), // BOTTOM-RIGHT
        ]
        .iter_mut()
        .map(|pos| pos.mul_element_wise(tex_coord_scale))
        .collect();

        Self {
            vertices,
            indices,
            tex_coords: whole_tex_coords.clone(),
            whole_tex_coords,
            texture_index,
            is_flipped: false,
            shader_type: ShaderType::STANDARD,
            z_value,
        }
    }

    pub fn textured_quad(texture_index: u32, z_value: f32) -> Self {
        Self::textured_quad_with_coords(texture_index, z_value, cgmath::Vector2::new(1., 1.))
    }

    pub fn circle(z_value: f32) -> Self {
        const NUM_TRIANGLES: u32 = 32;

        let center = cgmath::Vector2::new(0., 0.);
        let mut vertices = vec![center];
        let mut indices = vec![];

        for i in 0..NUM_TRIANGLES {
            let angle = (i as f64) * f64::consts::TAU / (NUM_TRIANGLES as f64);
            let point_x = angle.cos();
            let point_y = angle.sin();
            vertices.push(center + cgmath::Vector2::new(point_x as f32, point_y as f32));

            indices.push(0);
            indices.push((i + 1) % NUM_TRIANGLES + 1);
            indices.push(i + 1);
        }

        let whole_tex_coords: Vec<cgmath::Vector2<f32>> = vertices
            .iter()
            .map(|vertex| cgmath::Vector2::new(vertex.x, 1. - vertex.y))
            .collect();

        Self {
            vertices,
            indices,
            tex_coords: whole_tex_coords.clone(),
            whole_tex_coords,
            texture_index: 999,
            is_flipped: false,
            shader_type: ShaderType::STANDARD,
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

#[derive(Debug, Clone)]
pub struct PositionComponent {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
}

impl Component for PositionComponent {
    fn name(&self) -> String {
        return "Position".to_string();
    }
}

impl PositionComponent {
    pub fn scale_outward(&mut self, scale: cgmath::Vector2<f32>) {
        assert!(scale.x >= 1. && scale.y >= 1.);
        self.scale = self.scale.mul_element_wise(scale);
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub enum CharacterState {
    IDLE,
    MOVE,
    ATTACK,
    JUMPUP,
    JUMPDOWN,
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

// #[derive(Debug)]
// pub ControllableComponent {
// pub jump_available: bool,
// pub is_grounded:
// }

// impl Component for ControllableComponent {
//     fn name(&self) -> String {
//         return "Controllable".to_string();
//     }
// }

#[derive(Debug)]
pub struct MetadataComponent {
    // 00000000 00000000 00000000 00000jco
    // o -> should_outline
    // c -> is_controllable
    // j -> jump_available
    flags: u32,
}

impl Component for MetadataComponent {
    fn name(&self) -> String {
        return "Metadata".to_string();
    }
}

impl MetadataComponent {
    pub fn new(should_outline: bool, is_controllable: bool) -> Self {
        let mut flags: u32 = 0;
        if should_outline {
            flags |= 1;
        }
        if is_controllable {
            flags |= 2;
        }

        flags |= 1 << 2;

        flags |= 1 << 3;

        Self { flags }
    }

    pub fn should_outline(&self) -> bool {
        self.flags & 1 > 0
    }

    pub fn is_controllable(&self) -> bool {
        self.flags & 2 > 0
    }

    pub fn set_jump(&mut self, jump_available: bool) {
        if jump_available {
            self.flags |= 1 << 2
        } else {
            self.flags &= !(1 << 2)
        }
    }

    pub fn can_jump(&self) -> bool {
        self.flags & (1 << 2) > 0
    }
}

pub struct ParallaxComponent {
    pub move_speed: f32,
    pub layer: u32,
}

impl ParallaxComponent {}

pub struct CollectibleComponent {
    pub is_collected: bool,
    pub bounding_box: physics::BoundingBox,
    pub popup_text: &'static str,
}

pub struct SignComponent {
    pub in_range: bool,
    pub bounding_box: physics::BoundingBox,
    pub popup_text: &'static str,
}

#[derive(Debug)]
pub struct MovingPlatformComponent {
    pub amplitude: f32,
    pub period_secs: f32,
    pub original_position: cgmath::Vector2<f32>,
    pub horizontal: bool,
    pub prev_change: f32,
}

// pub fn update_platforms(
//     moving_platform_components: &mut EntityMap<MovingPlatformComponent>,
//     position_components: &mut EntityMap<PositionComponent>,
//     collider_box_components: &mut EntityMap<physics::ColliderBoxComponent>,
//     time_elapsed: Duration,
// ) {
//     zip3_entities_mut(
//         moving_platform_components,
//         position_components,
//         collider_box_components,
//     )
//     .for_each(|(_, moving_platform, position_component, collider_box)| {
//         if let (Some(moving_platform), Some(pos)) = (moving_platform, position_component) {
//             let change = (time_elapsed.as_secs_f32() * 2.0 * std::f32::consts::PI
//                 / moving_platform.period_secs)
//                 .sin()
//                 * moving_platform.amplitude;

//             if moving_platform.horizontal {
//                 pos.position.x = moving_platform.original_position.x + change;
//             } else {
//                 pos.position.y = moving_platform.original_position.y + change;
//             }

//             if let Some(collider_box) = collider_box {
//                 collider_box.bounding_box.bottom_left = pos.position - pos.scale / 2.0;
//                 collider_box.bounding_box.top_right = pos.position + pos.scale / 2.0;
//             }
//         }
//     });
// }
