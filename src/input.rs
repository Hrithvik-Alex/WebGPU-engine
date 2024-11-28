use crate::component::{self, PositionComponent};

use std::time::Duration;

use winit::{
    dpi::PhysicalPosition,
    event::*,
    keyboard::{KeyCode, PhysicalKey},
};

pub struct InputHandler {
    up_pressed: bool,
    down_pressed: bool,
    left_pressed: bool,
    right_pressed: bool,
    mouse_position: PhysicalPosition<f64>,
}

impl InputHandler {
    const MOVEMENT_SPEED: f32 = 50.;

    pub fn new() -> Self {
        Self {
            up_pressed: false,
            down_pressed: false,
            left_pressed: false,
            right_pressed: false,
            mouse_position: PhysicalPosition::new(0., 0.),
        }
    }
    pub fn handle_key_state(&mut self, event: &KeyEvent) {
        match event.state {
            ElementState::Pressed => match event.physical_key {
                PhysicalKey::Code(KeyCode::Space) => (),
                PhysicalKey::Code(KeyCode::KeyW) | PhysicalKey::Code(KeyCode::ArrowUp) => {
                    self.up_pressed = true
                }
                PhysicalKey::Code(KeyCode::KeyA) | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                    self.left_pressed = true
                }
                PhysicalKey::Code(KeyCode::KeyS) | PhysicalKey::Code(KeyCode::ArrowDown) => {
                    self.down_pressed = true
                }
                PhysicalKey::Code(KeyCode::KeyD) | PhysicalKey::Code(KeyCode::ArrowRight) => {
                    self.right_pressed = true
                }
                _ => (),
            },

            ElementState::Released => match event.physical_key {
                PhysicalKey::Code(KeyCode::KeyW) | PhysicalKey::Code(KeyCode::ArrowUp) => {
                    self.up_pressed = false
                }
                PhysicalKey::Code(KeyCode::KeyA) | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                    self.left_pressed = false
                }
                PhysicalKey::Code(KeyCode::KeyS) | PhysicalKey::Code(KeyCode::ArrowDown) => {
                    self.down_pressed = false
                }
                PhysicalKey::Code(KeyCode::KeyD) | PhysicalKey::Code(KeyCode::ArrowRight) => {
                    self.right_pressed = false
                }
                _ => (),
            },
        }
    }

    pub fn update_state(
        &self,
        position_components: &mut component::EntityMap<PositionComponent>,
        delta_time: Duration,
    ) {
        let mut update_position = |x: f32, y: f32| {
            let delta =
                cgmath::Vector2::new(x, y) * Self::MOVEMENT_SPEED * delta_time.as_secs_f32();
            position_components
                .iter_mut()
                .for_each(|(_, position_component)| match position_component {
                    Some(position_component) if position_component.is_controllable => {
                        position_component.position += delta;
                    }
                    _ => (),
                });
            // let position = state.sprite.get_position();

            // state.sprite.update_position(position + delta)
        };
        if self.up_pressed {
            update_position(0., 1.)
        }
        if self.down_pressed {
            update_position(0., -1.)
        }
        if self.left_pressed {
            update_position(-1., 0.)
        }
        if self.right_pressed {
            update_position(1., 0.)
        }
    }

    pub fn set_position(&mut self, position: PhysicalPosition<f64>) {
        self.mouse_position = position
    }
}

// TODO: make key press a flag so that movement can happen per frame
