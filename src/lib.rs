mod animation;
mod camera;
mod component;
mod context;
mod game;
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
mod wgsl_preprocessor;

use egui_winit::winit;
use egui_winit::winit::{
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Theme, Window, WindowAttributes, WindowId},
};

use instant::Instant;
use log::debug;

use physics::ColliderBoxComponent;
use state::State;
use std::sync::{Arc, Mutex};
use std::thread::current;
use std::time::Duration;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use winit::application::ApplicationHandler;
use winit::event;
use winit::event_loop::EventLoopProxy;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

enum UserEvent {
    StateReady(state::State<'static>, Arc<Window>),
}

pub struct App {
    window: Option<Arc<Window>>,
    state: Option<state::State<'static>>,
    event_loop_proxy: EventLoopProxy<UserEvent>,
    player: Option<component::Entity>,

    last_fps: u32,
    frames: u32,
    start_time: Instant,
    seconds_elapsed: u64,
    last_frame_time: Duration,
    ticks_elapsed: Duration,
}

impl App {
    pub fn new(event_loop: &EventLoop<UserEvent>) -> Self {
        let start_time = Instant::now();
        let frames = 0;
        let seconds_elapsed = 0;
        let last_frame_time = Duration::new(0, 0);

        let ticks_elapsed = Duration::new(0, 0);

        Self {
            window: None,
            state: None,
            event_loop_proxy: event_loop.create_proxy(),
            player: None,
            start_time,
            last_fps: 0,
            frames,
            seconds_elapsed,
            last_frame_time,
            ticks_elapsed,
        }
    }

    fn init_state(&mut self, mut state: State<'static>, window: Arc<Window>) {
        let player = state.init();
        self.window = Some(window);
        self.state = Some(state);
        self.player = Some(player);
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                std::panic::set_hook(Box::new(console_error_panic_hook::hook));
                console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize logger");

                debug!("It works!");
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

        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::Element;
            use winit::{dpi::PhysicalSize, platform::web::WindowExtWebSys};

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-game")?;
                    let canvas = Element::from(window.canvas()?);
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");

            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            let _ = window.request_inner_size(PhysicalSize::new(768, 500));

            let state_future = State::new(window.clone());
            let event_loop_proxy = self.event_loop_proxy.clone();

            let state = async move {
                let state = state_future.await;
                assert!(event_loop_proxy
                    .send_event(UserEvent::StateReady(state, window.clone()))
                    .is_ok());
            };

            wasm_bindgen_futures::spawn_local(state)
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let state = pollster::block_on(State::new(window.clone()));
            // self.init_state(state, window);
            assert!(self
                .event_loop_proxy
                .send_event(UserEvent::StateReady(state, window))
                .is_ok());
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        let UserEvent::StateReady(state, window) = event;
        self.init_state(state, window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let (Some(ref mut state), Some(player)) = (&mut self.state, self.player) {
            self.frames += 1;
            let current_time = self.start_time.elapsed();
            let delta_time = current_time - self.last_frame_time;
            if current_time > Duration::new(self.seconds_elapsed + 1, 0) {
                self.last_fps = self.frames;
                debug!("FPS {:?}", self.last_fps);
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
                    &mut state.physics_components,
                    &mut state.collectible_components,
                    &mut state.sign_components,
                    &mut state.moving_platform_components,
                    &mut state.character_state_components,
                    current_time,
                    &state.game_mode,
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

            let player_position = state.position_components.get(player);
            assert!(player_position.is_some() && player_position.unwrap().is_some());
            camera::CameraController::update(
                // &state.context,
                player_position.unwrap().as_ref().unwrap().position,
                &mut state.camera,
                &state.world_uniform,
                &mut state.parallax_components,
                &mut state.vertex_array_components,
                &mut state.position_components,
            );

            state.update_platformer_game_state();

            state.gui_info.fps = self.last_fps as u32;

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
                    WindowEvent::CloseRequested => event_loop.exit(),
                    WindowEvent::Resized(physical_size) => state.resize(physical_size),

                    WindowEvent::RedrawRequested => {
                        let render_result = state.render_system.render(
                            &state.position_components,
                            &state.vertex_array_components,
                            &state.light_components,
                            &state.metadata_components,
                            &state.sprite_sheets,
                            &state.context,
                            &mut state.gui,
                            state.window.clone(),
                            false,
                            current_time,
                            &state.world_uniform,
                            &state.camera,
                            &state.gui_info,
                            &mut state.game_mode,
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
                        &mut state.sign_components,
                        &mut state.game_mode,
                        &mut state.gui_info,
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::DEBUG.into())
        .from_env_lossy()
        .add_directive("wgpu_core::device::resource=warn".parse().unwrap());
    let subscriber = tracing_subscriber::registry().with(env_filter);
    #[cfg(target_arch = "wasm32")]
    {
        use tracing_wasm::{WASMLayer, WASMLayerConfig};

        console_error_panic_hook::set_once();
        let wasm_layer = WASMLayer::new(WASMLayerConfig::default());

        subscriber.with(wasm_layer).init();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let fmt_layer = tracing_subscriber::fmt::Layer::default();
        subscriber.with(fmt_layer).try_init();
    }

    let event_loop = EventLoop::with_user_event().build().unwrap();
    let mut app = App::new(&event_loop);
    event_loop.run_app(&mut app).unwrap();
}
