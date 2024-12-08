mod animation;
mod camera;
mod component;
mod context;
mod input;
mod model;
mod physics;
mod render_system;
mod sprite;
mod state;
mod texture;

use log::debug;
use physics::ColliderBoxComponent;
use std::time::{Duration, Instant};
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }
    let mut state = state::State::new(&window).await;
    let mut input_handler = input::InputHandler::new();

    let textures = state.textures();

    let render_system = render_system::RenderSystem::new(
        &textures,
        &state.context,
        &state.world_uniform,
        &state.camera,
    );

    let ground = {
        let position_component = component::PositionComponent {
            position: cgmath::Vector2::new(0., 0.),
            scale: cgmath::Vector2::new(640., 100.),
            is_controllable: false,
        };

        let vertex_array_component = component::VertexArrayComponent::textured_quad(999);

        let collider_box_component = ColliderBoxComponent {
            bottom_left: position_component.position,
            top_right: position_component.position + position_component.scale,
        };

        state.add_entity(
            Some(position_component),
            Some(vertex_array_component),
            None,
            None,
            None,
            Some(collider_box_component),
        )
    };
    // entity for player
    let character = {
        let position_component = component::PositionComponent {
            position: cgmath::Vector2::new(50., 100.),
            scale: cgmath::Vector2::new(64., 64.),
            is_controllable: true,
        };

        let texture_index = 0; // warrior

        let vertex_array_component = component::VertexArrayComponent::textured_quad(texture_index);

        let sprite_animation_idle = animation::SpriteAnimation {
            animation_index: 0,
            sprite_count: 10,
            start_index: 0,
            per_sprite_duration: Duration::new(0, 125000000),
            current_elapsed_time: Duration::new(0, 0),
        };
        let sprite_animation_run = animation::SpriteAnimation {
            animation_index: 0,
            sprite_count: 10,
            start_index: 20,
            per_sprite_duration: Duration::new(0, 125000000),
            current_elapsed_time: Duration::new(0, 0),
        };
        let sprite_animation_attack = animation::SpriteAnimation {
            animation_index: 0,
            sprite_count: 10,
            start_index: 30,
            per_sprite_duration: Duration::new(0, 125000000),
            current_elapsed_time: Duration::new(0, 0),
        };

        let mut sprite_animation_controller = animation::SpriteAnimationControllerComponent::new();
        sprite_animation_controller
            .animation_map
            .insert(component::CharacterState::IDLE, sprite_animation_idle);
        sprite_animation_controller
            .animation_map
            .insert(component::CharacterState::MOVE, sprite_animation_run);
        sprite_animation_controller
            .animation_map
            .insert(component::CharacterState::ATTACK, sprite_animation_attack);

        let sheet_position_component = sprite::SheetPositionComponent {
            sprite_sheet: state.sprite_sheets[texture_index as usize].clone(),
            sheet_position: cgmath::Vector2::new(0, 0),
        };

        let character_state_component = component::CharacterStateComponent {
            character_state: component::CharacterState::IDLE,
        };

        let collider_box_component = ColliderBoxComponent {
            bottom_left: position_component.position,
            top_right: position_component.position + position_component.scale,
        };

        state.add_entity(
            Some(position_component),
            Some(vertex_array_component),
            Some(sprite_animation_controller),
            Some(sheet_position_component),
            Some(character_state_component),
            Some(collider_box_component),
        )
    };

    let minotaur = {
        let position_component = component::PositionComponent {
            position: cgmath::Vector2::new(200., 100.),
            scale: cgmath::Vector2::new(64., 64.),
            is_controllable: false,
        };

        let texture_index = 1; // warrior

        let vertex_array_component = component::VertexArrayComponent::textured_quad(texture_index);
        let sprite_animation_idle = animation::SpriteAnimation {
            animation_index: 0,
            sprite_count: 10,
            start_index: 0,
            per_sprite_duration: Duration::new(0, 125000000),
            current_elapsed_time: Duration::new(0, 0),
        };
        let sprite_animation_run = animation::SpriteAnimation {
            animation_index: 0,
            sprite_count: 10,
            start_index: 20,
            per_sprite_duration: Duration::new(0, 125000000),
            current_elapsed_time: Duration::new(0, 0),
        };
        let sprite_animation_attack = animation::SpriteAnimation {
            animation_index: 0,
            sprite_count: 10,
            start_index: 30,
            per_sprite_duration: Duration::new(0, 125000000),
            current_elapsed_time: Duration::new(0, 0),
        };

        let mut sprite_animation_controller = animation::SpriteAnimationControllerComponent::new();
        sprite_animation_controller
            .animation_map
            .insert(component::CharacterState::IDLE, sprite_animation_idle);
        sprite_animation_controller
            .animation_map
            .insert(component::CharacterState::MOVE, sprite_animation_run);
        sprite_animation_controller
            .animation_map
            .insert(component::CharacterState::ATTACK, sprite_animation_attack);

        let sheet_position_component = sprite::SheetPositionComponent {
            sprite_sheet: state.sprite_sheets[texture_index as usize].clone(),
            sheet_position: cgmath::Vector2::new(0, 0),
        };

        let character_state_component = component::CharacterStateComponent {
            character_state: component::CharacterState::IDLE,
        };

        let collider_box_component = ColliderBoxComponent {
            bottom_left: position_component.position,
            top_right: position_component.position + position_component.scale,
        };

        state.add_entity(
            Some(position_component),
            Some(vertex_array_component),
            Some(sprite_animation_controller),
            Some(sheet_position_component),
            Some(character_state_component),
            Some(collider_box_component),
        )
    };

    debug!("{:?}", state.vertex_array_components);

    let start_time = Instant::now();
    let mut frames = 0;
    let mut seconds_elapsed: u64 = 0;
    let mut last_frame_time: Duration = Duration::new(0, 0);

    const FIXED_UPDATES_PER_SECOND: u32 = 50;
    const FIXED_UPDATE_DURATION: Duration = Duration::new(0, 1000000000 / FIXED_UPDATES_PER_SECOND);
    let mut ticks_elapsed = Duration::new(0, 0);

    let physics_system = physics::PhysicsSystem::new(FIXED_UPDATE_DURATION);

    let _ = event_loop.run(move |event, control_flow| {
        {
            frames += 1;
            let current_time = start_time.elapsed();
            let delta_time = current_time - last_frame_time;
            if current_time > Duration::new(seconds_elapsed + 1, 0) {
                debug!("FPS {:?}", frames);
                frames = 0;
                seconds_elapsed += 1;
            }
            last_frame_time = current_time;

            ticks_elapsed += delta_time;
            while ticks_elapsed > FIXED_UPDATE_DURATION {
                physics_system.update(
                    &input_handler,
                    &mut state.position_components,
                    &mut state.collider_box_components,
                );

                ticks_elapsed -= FIXED_UPDATE_DURATION;
            }

            sprite::SpriteSheetSystem::update(
                &mut state.vertex_array_components,
                &state.sheet_position_components,
            );
            animation::AnimationSystem::update_animations(
                &mut state.sprite_animation_controller_components,
                &mut state.sheet_position_components,
                &state.character_state_components,
                delta_time,
            );

            // idle_anim.update(delta_time);
            // state
            //     .sprite
            //     .update_sheet_position(idle_anim.get_sheet_index());
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == state.window.id() => {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => state.resize(*physical_size),

                        WindowEvent::RedrawRequested => {
                            let render_result = render_system.render(
                                &state.position_components,
                                &state.vertex_array_components,
                                &textures,
                                &state.context,
                                true,
                            );
                            match render_result {
                                Ok(_) => {}
                                // Reconfigure the surface if lost
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    state.resize(state.size)
                                }
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                                // All other errors (Outdated, Timeout) should be resolved by the next frame
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }

                        WindowEvent::CursorMoved {
                            device_id: _,
                            position,
                        } => input_handler.set_position(*position),
                        WindowEvent::KeyboardInput {
                            device_id: _,
                            event,
                            is_synthetic: _,
                        } => input_handler.handle_key_state(
                            event,
                            &mut state.position_components,
                            &mut state.character_state_components,
                            &mut state.vertex_array_components,
                        ),

                        _ => {}
                    }
                }

                Event::AboutToWait => {
                    // RedrawRequested will only trigger once unless we manually
                    // request it.
                    state.window().request_redraw();
                }
                _ => {}
            }
        };
    });
}
