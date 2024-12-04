use std::{collections::HashMap, time::Duration};

use crate::{component, sprite::SheetPositionComponent};

pub struct SpriteAnimation {
    pub animation_index: u32,
    pub sprite_count: u32,
    pub start_index: u32,
    pub per_sprite_duration: Duration,
    pub current_elapsed_time: Duration,
}

impl SpriteAnimation {
    pub fn update(&mut self, delta_time: Duration) {
        self.current_elapsed_time += delta_time;
        if self.current_elapsed_time > self.per_sprite_duration {
            self.current_elapsed_time -= self.per_sprite_duration;
            self.animation_index = (self.animation_index + 1) % self.sprite_count;
        }
    }

    pub fn get_sheet_index(&self) -> u32 {
        self.start_index + self.animation_index
    }
}
pub struct SpriteAnimationControllerComponent {
    pub animation_map: HashMap<component::CharacterState, SpriteAnimation>,
}

impl component::Component for SpriteAnimationControllerComponent {
    fn name(&self) -> String {
        "SpriteAnimationController".to_string()
    }
}

impl SpriteAnimationControllerComponent {
    pub fn new() -> Self {
        Self {
            animation_map: HashMap::new(),
        }
    }
}

pub struct AnimationSystem {}

impl AnimationSystem {
    pub fn update_animations(
        sprite_animation_controller_components: &mut component::EntityMap<
            SpriteAnimationControllerComponent,
        >,
        sheet_position_components: &mut component::EntityMap<SheetPositionComponent>,
        character_state_components: &component::EntityMap<component::CharacterStateComponent>,
        delta_time: Duration,
    ) {
        sprite_animation_controller_components
            .iter_mut()
            .zip(sheet_position_components.iter_mut())
            .zip(character_state_components.iter())
            .for_each(
                |(
                    ((_, sprite_animation_controller), (_, sheet_position_component)),
                    (_, character_state_component),
                )| {
                    if let (
                        Some(sprite_animation_controller),
                        Some(sheet_position_component),
                        Some(character_state_component),
                    ) = (
                        sprite_animation_controller,
                        sheet_position_component,
                        character_state_component,
                    ) {
                        let sprite_animation = sprite_animation_controller
                            .animation_map
                            .get_mut(&character_state_component.character_state);
                        if let Some(sprite_animation) = sprite_animation {
                            sprite_animation.update(delta_time);

                            sheet_position_component.sheet_position = sheet_position_component
                                .sprite_sheet
                                .get_position_by_index(sprite_animation.get_sheet_index());
                        }
                    }
                },
            );
    }
}
