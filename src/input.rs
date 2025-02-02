use crate::{
    component::{self, PositionComponent},
    game, gui, utils,
};

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
        metadata_components: &mut component::EntityMap<component::MetadataComponent>,
        sign_components: &mut component::EntityMap<component::SignComponent>,
        game_mode: &mut game::GameMode,
        gui_info: &mut gui::GuiInfo,
    ) {
        let mut update_state = |state: component::CharacterState, is_flipped: Option<bool>| {
            utils::zip4_entities_mut(
                position_components,
                character_state_components,
                vertex_array_components,
                metadata_components,
            )
            .for_each(
                |(
                    _,
                    position_component,
                    character_state_component,
                    vertex_array_component,
                    metadata_component,
                )| {
                    let metadata_component = metadata_component.as_ref().unwrap();
                    match position_component {
                        Some(position_component) if metadata_component.is_controllable() => {
                            if let Some(character_state_component) =
                                character_state_component.as_mut()
                            {
                                if metadata_component.can_jump() {
                                    character_state_component.character_state = state.clone();
                                }
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

        match *game_mode {
            game::GameMode::POPUP => {
                update_state(component::CharacterState::IDLE, None);
                self.up_pressed = false;
                self.left_pressed = false;
                self.right_pressed = false;
                self.down_pressed = false;

                match event.state {
                    ElementState::Pressed => match event.physical_key {
                        PhysicalKey::Code(KeyCode::Escape) => *game_mode = game::GameMode::STANDARD,
                        _ => (),
                    },
                    _ => (),
                }
            }
            game::GameMode::STANDARD => {
                match event.state {
                    ElementState::Pressed => match event.physical_key {
                        PhysicalKey::Code(KeyCode::Space) => (),
                        PhysicalKey::Code(KeyCode::KeyW) | PhysicalKey::Code(KeyCode::ArrowUp) => {
                            self.up_pressed = true
                        }
                        PhysicalKey::Code(KeyCode::KeyA)
                        | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                            self.left_pressed = true;
                            update_state(component::CharacterState::MOVE, Some(false));
                        }
                        PhysicalKey::Code(KeyCode::KeyS)
                        | PhysicalKey::Code(KeyCode::ArrowDown) => self.down_pressed = true,
                        PhysicalKey::Code(KeyCode::KeyD)
                        | PhysicalKey::Code(KeyCode::ArrowRight) => {
                            self.right_pressed = true;
                            update_state(component::CharacterState::MOVE, Some(true));
                        }
                        PhysicalKey::Code(KeyCode::KeyX) => {
                            sign_components.iter_mut().for_each(|(_, sign)| {
                                sign.as_mut().map(|sign| {
                                    if sign.in_range {
                                        gui_info.popup_text = sign.popup_text;
                                        gui_info.popup_type = gui::PopupType::WOOD;
                                        *game_mode = game::GameMode::POPUP;
                                    }
                                });
                            });
                        }
                        _ => (),
                    },

                    ElementState::Released => match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyW) | PhysicalKey::Code(KeyCode::ArrowUp) => {
                            self.up_pressed = false
                        }
                        PhysicalKey::Code(KeyCode::KeyA)
                        | PhysicalKey::Code(KeyCode::ArrowLeft) => {
                            self.left_pressed = false;
                        }
                        PhysicalKey::Code(KeyCode::KeyS)
                        | PhysicalKey::Code(KeyCode::ArrowDown) => self.down_pressed = false,
                        PhysicalKey::Code(KeyCode::KeyD)
                        | PhysicalKey::Code(KeyCode::ArrowRight) => {
                            self.right_pressed = false;
                        }
                        _ => (),
                    },
                }

                if !self.left_pressed && !self.right_pressed {
                    update_state(component::CharacterState::IDLE, None);
                }
            }
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
