mod animation;
mod camera;
mod component;
mod context;
mod input;
mod model;
mod render_system;
mod sprite;
mod state;
mod texture;

use log::debug;
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

    let start_time = Instant::now();
    let mut frames = 0;
    let mut seconds_elapsed: u64 = 0;
    let mut last_frame_time: Duration = Duration::new(0, 0);

    let textures = state.textures();

    let render_system = render_system::RenderSystem::new(
        &textures,
        &state.context,
        &state.world_uniform,
        &state.camera,
    );

    // entity for player
    let character = {
        let position_component = component::PositionComponent {
            position: cgmath::Vector2::new(50., 100.),
            scale: 64.,
            is_controllable: true,
        };

        let texture_index = 0; // warrior

        let vertex_array_component = component::VertexArrayComponent::textured_quad(texture_index);
        let sprite_animation = animation::SpriteAnimation {
            animation_index: 0,
            sprite_count: 10,
            start_index: 0,
            per_sprite_duration: Duration::new(0, 125000000),
            current_elapsed_time: Duration::new(0, 0),
        };

        let sheet_position_component = sprite::SheetPositionComponent {
            sprite_sheet: state.sprite_sheets[texture_index as usize].clone(),
            sheet_position: cgmath::Vector2::new(0, 0),
        };

        state.add_entity(
            Some(position_component),
            Some(vertex_array_component),
            Some(sprite_animation),
            Some(sheet_position_component),
        )
    };

    let minotaur = {
        let position_component = component::PositionComponent {
            position: cgmath::Vector2::new(200., 100.),
            scale: 64.,
            is_controllable: false,
        };

        let texture_index = 1; // warrior

        let vertex_array_component = component::VertexArrayComponent::textured_quad(texture_index);
        let sprite_animation = animation::SpriteAnimation {
            animation_index: 0,
            sprite_count: 10,
            start_index: 0,
            per_sprite_duration: Duration::new(0, 125000000),
            current_elapsed_time: Duration::new(0, 0),
        };

        let sheet_position_component = sprite::SheetPositionComponent {
            sprite_sheet: state.sprite_sheets[texture_index as usize].clone(),
            sheet_position: cgmath::Vector2::new(0, 0),
        };

        state.add_entity(
            Some(position_component),
            Some(vertex_array_component),
            Some(sprite_animation),
            Some(sheet_position_component),
        )
    };

    debug!("{:?}", state.vertex_array_components);

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

            animation::AnimationSystem::update_animations(
                &mut state.sprite_animation_components,
                &mut state.sheet_position_components,
                delta_time,
            );

            input_handler.update_state(&mut state.position_components, delta_time);
            sprite::SpriteSheetSystem::update(
                &mut state.vertex_array_components,
                &state.sheet_position_components,
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
                        // TODO: refactor input controller
                        WindowEvent::KeyboardInput {
                            device_id: _,
                            event,
                            is_synthetic: _,
                        } => input_handler.handle_key_state(event),

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
