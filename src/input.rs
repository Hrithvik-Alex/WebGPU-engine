use crate::component::{self, CharacterStateComponent, PositionComponent};

use std::time::Duration;

use log::debug;
use winit::{
    dpi::PhysicalPosition,
    event::*,
    keyboard::{KeyCode, PhysicalKey},
};

pub struct InputHandler {
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    mouse_position: PhysicalPosition<f64>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            up_pressed: false,
            down_pressed: false,
            left_pressed: false,
            right_pressed: false,
            mouse_position: PhysicalPosition::new(0., 0.),
        }
    }
    pub fn handle_key_state(
        &mut self,
        event: &KeyEvent,
        position_components: &mut component::EntityMap<PositionComponent>,
        character_state_components: &mut component::EntityMap<component::CharacterStateComponent>,
        vertex_array_components: &mut component::EntityMap<component::VertexArrayComponent>,
    ) {
        let mut update_state = |state: component::CharacterState, is_flipped: Option<bool>| {
            debug!("{:?}", state);
            position_components
                .iter_mut()
                .zip(character_state_components.iter_mut())
                .zip(vertex_array_components.iter_mut())
                .for_each(
                    |(
                        ((_, position_component), (_, character_state_component)),
                        (_, vertex_array_component),
                    )| {
                        match position_component {
                            Some(position_component) if position_component.is_controllable => {
                                if let Some(character_state_component) =
                                    character_state_component.as_mut()
                                {
                                    character_state_component.character_state = state.clone();
                                }

                                if let (Some(is_flipped), Some(vertex_array_component)) =
                                    (is_flipped, vertex_array_component)
                                {
                                    vertex_array_component.is_flipped = is_flipped;
                                }
                            }
                            _ => (),
                        }
                    },
                );
            // let position = state.sprite.get_position();

            // state.sprite.update_position(position + delta)
        };

        match event.state {
            ElementState::Pressed => match event.physical_key {
                PhysicalKey::Code(KeyCode::Space) => (),
                PhysicalKey::Code(KeyCode::KeyW) | PhysicalKey::Code(KeyCode::ArrowUp) => {
                    self.up_pressed = true
                }
                PhysicalKey::Code(KeyCode::KeyA) | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                    self.left_pressed = true;
                    update_state(component::CharacterState::MOVE, Some(true));
                }
                PhysicalKey::Code(KeyCode::KeyS) | PhysicalKey::Code(KeyCode::ArrowDown) => {
                    self.down_pressed = true
                }
                PhysicalKey::Code(KeyCode::KeyD) | PhysicalKey::Code(KeyCode::ArrowRight) => {
                    self.right_pressed = true;
                    update_state(component::CharacterState::MOVE, Some(false));
                }
                _ => (),
            },

            ElementState::Released => match event.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) | PhysicalKey::Code(KeyCode::ArrowUp) => {
                    self.up_pressed = false
                }
                PhysicalKey::Code(KeyCode::KeyA) | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                    self.left_pressed = false;
                }
                PhysicalKey::Code(KeyCode::KeyS) | PhysicalKey::Code(KeyCode::ArrowDown) => {
                    self.down_pressed = false
                }
                PhysicalKey::Code(KeyCode::KeyD) | PhysicalKey::Code(KeyCode::ArrowRight) => {
                    self.right_pressed = false;
                }
                _ => (),
            },
        }

        if !self.left_pressed && !self.right_pressed {
            update_state(component::CharacterState::IDLE, None);
        }
    }

    // pub fn update_state(
    //     &self,
    //     position_components: &mut component::EntityMap<PositionComponent>,
    //     character_state_components: &mut component::EntityMap<component::CharacterStateComponent>,
    //     delta_time: Duration,
    // ) {

    // }

    pub fn set_position(&mut self, position: PhysicalPosition<f64>) {
        self.mouse_position = position
    }
}
