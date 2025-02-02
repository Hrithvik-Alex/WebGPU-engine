use crate::{
    component::{self, EntityMap},
    physics, utils,
};

pub struct PlatformerGameState {
    pub notes_collected: u32,
    pub character_init_position: cgmath::Vector2<f32>,
}

impl PlatformerGameState {
    pub fn new(character_init_position: cgmath::Vector2<f32>) -> Self {
        Self {
            notes_collected: 0,
            character_init_position,
        }
    }

    pub fn update(
        &mut self,
        position_components: &mut EntityMap<component::PositionComponent>,
        collider_box_components: &mut EntityMap<physics::ColliderBoxComponent>,
        metadata_components: &mut EntityMap<component::MetadataComponent>,
    ) {
        utils::zip3_entities_mut(
            position_components,
            collider_box_components,
            metadata_components,
        )
        .for_each(|(_, pos, collider, metadata)| {
            if metadata.as_ref().unwrap().is_controllable() {
                assert!(pos.is_some());
                if let Some(pos) = pos {
                    if pos.position.y < 0. {
                        pos.position = self.character_init_position;
                        if let Some(collider_box) = collider {
                            collider_box.bounding_box.update(pos.position);
                        }
                    }
                }
            }
        });
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum GameMode {
    STANDARD,
    POPUP,
}
