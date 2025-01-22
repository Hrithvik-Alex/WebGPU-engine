use crate::{
    component::{self, Entity, EntityMap},
    physics, state, utils,
};

pub struct MiraGameState {
    pub notes_collected: u32,
    pub mira_init_position: cgmath::Vector2<f32>,
}

impl MiraGameState {
    pub fn new(mira_init_position: cgmath::Vector2<f32>) -> Self {
        Self {
            notes_collected: 0,
            mira_init_position,
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
                        pos.position = self.mira_init_position;
                        if let Some(collider_box) = collider {
                            collider_box.bounding_box.bottom_left = pos.position - pos.scale / 2.0;
                            collider_box.bounding_box.top_right = pos.position + pos.scale / 2.0;
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
