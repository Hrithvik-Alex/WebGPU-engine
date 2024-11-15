mod camera;
mod context;
mod input;
mod model;
mod sprite;
mod state;
mod texture;

use cgmath::{SquareMatrix, Vector2};
use log::debug;
use model::Vertex;
use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalPosition,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use winit::window::Window;

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

    let mut idle_anim = sprite::SpriteAnimation {
        animation_index: 0,
        sprite_count: 8,
        start_index: 0,
        per_sprite_duration: Duration::new(0, 125000000),
        current_elapsed_time: Duration::new(0, 0),
    };

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
            input_handler.update_state(&mut state, delta_time);
            idle_anim.update(delta_time);
            state
                .sprite
                .update_sheet_position(idle_anim.get_sheet_index());
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == state.window.id() => {
                    if !state.input(event) {
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
                                state.update();
                                match state.render() {
                                    Ok(_) => {}
                                    // Reconfigure the surface if lost
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => state.resize(state.size),
                                    // The system is out of memory, we should probably quit
                                    Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                                    Err(e) => eprintln!("{:?}", e),
                                }
                            }

                            WindowEvent::CursorMoved {
                                device_id: _,
                                position,
                            } => state.set_position(*position),
                            // TODO: refactor input controller
                            WindowEvent::KeyboardInput {
                                device_id: _,
                                event,
                                is_synthetic: _,
                            } => input_handler.handle_key_state(event),

                            _ => {}
                        }
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
