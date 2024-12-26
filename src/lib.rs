mod animation;
mod camera;
mod component;
mod context;
mod gui;
mod input;
mod model;
mod physics;
mod render_system;
mod sprite;
mod state;
mod texture;
mod uniform;
mod utils;

use egui_winit::winit;
use egui_winit::winit::{
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Theme, Window, WindowAttributes, WindowId},
};

use log::debug;
use physics::ColliderBoxComponent;
use state::State;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct App<'a> {
    window: Option<Arc<Window>>,
    state: Option<state::State<'a>>,

    frames: i32,
    start_time: Instant,
    seconds_elapsed: u64,
    last_frame_time: Duration,
    ticks_elapsed: Duration,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let start_time = Instant::now();
        let frames = 0;
        let seconds_elapsed = 0;
        let last_frame_time = Duration::new(0, 0);

        let ticks_elapsed = Duration::new(0, 0);

        Self {
            window: None,
            state: None,
            start_time,
            frames,
            seconds_elapsed,
            last_frame_time,
            ticks_elapsed,
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
            } else {
                env_logger::init();
            }
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(winit::dpi::LogicalSize::new(768, 500)),
                )
                .unwrap(),
        );
        let mut state = pollster::block_on(state::State::new(window.clone())); // (1)
        state.init();
        self.window = Some(window); // (2)
        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(ref mut state) = &mut self.state {
            self.frames += 1;
            let current_time = self.start_time.elapsed();
            let delta_time = current_time - self.last_frame_time;
            if current_time > Duration::new(self.seconds_elapsed + 1, 0) {
                debug!("FPS {:?}", self.frames);
                self.frames = 0;
                self.seconds_elapsed += 1;
            }
            self.last_frame_time = current_time;

            self.ticks_elapsed += delta_time;
            while self.ticks_elapsed > state::State::FIXED_UPDATE_DURATION {
                state.physics_system.update(
                    &state.input_handler,
                    &mut state.position_components,
                    &mut state.collider_box_components,
                    &mut state.metadata_components,
                );

                self.ticks_elapsed -= state::State::FIXED_UPDATE_DURATION;
            }

            sprite::SpriteSheetSystem::update(
                &mut state.vertex_array_components,
                &state.sheet_position_components,
            );
            animation::AnimationSystem::update_animations(
                &mut state.sprite_animation_controller_components,
                &mut state.sheet_position_components,
                &mut state.character_state_components,
                delta_time,
            );

            if state
                .gui
                .state
                .on_window_event(&state.window, &event)
                .consumed
            {
                return;
            }
            // state.gui.state.on_window_event(&state.window, &event);
            // idle_anim.update(delta_time);
            // state
            //     .sprite
            //     .update_sheet_position(idle_anim.get_sheet_index());

            let textures = state.textures();
            if window_id == state.window.id() {
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
                    } => event_loop.exit(),
                    WindowEvent::Resized(physical_size) => state.resize(physical_size),

                    WindowEvent::RedrawRequested => {
                        let render_result = state.render_system.render(
                            &state.position_components,
                            &state.vertex_array_components,
                            &state.light_components,
                            &state.metadata_components,
                            &textures,
                            &state.context,
                            &mut state.gui,
                            state.window.clone(),
                            true,
                            current_time,
                        );
                        match render_result {
                            Ok(_) => {}
                            // Reconfigure the surface if lost
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                state.resize(state.size)
                            }
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }

                    WindowEvent::CursorMoved {
                        device_id: _,
                        position,
                    } => state.input_handler.set_position(position),
                    WindowEvent::KeyboardInput {
                        device_id: _,
                        event,
                        is_synthetic: _,
                    } => state.input_handler.handle_key_state(
                        &event,
                        &mut state.position_components,
                        &mut state.character_state_components,
                        &mut state.vertex_array_components,
                        &mut state.metadata_components,
                    ),

                    _ => {}
                }
            }

            // Event::AboutToWait => {
            //     // RedrawRequested will only trigger once unless we manually
            //     // request it.
            //     state.window().request_redraw();
            // }
            // _ => {}
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(ref mut window) = &mut self.window {
            window.request_redraw();
        }
    }
}

pub fn run() {
    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
