use std::time::Duration;

use crate::{component, sprite::SheetPositionComponent};

pub struct SpriteAnimation {
    pub animation_index: u32,
    pub sprite_count: u32,
    pub start_index: u32,
    pub per_sprite_duration: Duration,
    pub current_elapsed_time: Duration,
}

impl component::Component for SpriteAnimation {
    fn name(&self) -> String {
        "SpriteAnimation".to_string()
    }
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

pub struct AnimationSystem {}

impl AnimationSystem {
    pub fn update_animations(
        sprite_animation_components: &mut component::EntityMap<SpriteAnimation>,
        sheet_position_components: &mut component::EntityMap<SheetPositionComponent>,
        delta_time: Duration,
    ) {
        sprite_animation_components
            .iter_mut()
            .zip(sheet_position_components.iter_mut())
            .for_each(|((_, sprite_animation), (_, sheet_position_component))| {
                if let (Some(sprite_animation), Some(sheet_position_component)) =
                    (sprite_animation, sheet_position_component)
                {
                    sprite_animation.update(delta_time);

                    sheet_position_component.sheet_position = sheet_position_component
                        .sprite_sheet
                        .get_position_by_index(sprite_animation.get_sheet_index());
                }
            });
    }
}
