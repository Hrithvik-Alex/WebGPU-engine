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
        debug!("P U");
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
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
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
                &state.character_state_components,
                delta_time,
            );

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
                            &textures,
                            &state.context,
                            &state.gui,
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

// #[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
// pub async fn run() {
//     cfg_if::cfg_if! {
//         if #[cfg(target_arch = "wasm32")] {
//             std::panic::set_hook(Box::new(console_error_panic_hook::hook));
//             console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
//         } else {
//             env_logger::init();
//         }
//     }

//     let event_loop = EventLoop::new().unwrap();
//     let window = WindowAttributes::new().build(&event_loop).unwrap();

//     #[cfg(target_arch = "wasm32")]
//     {
//         // Winit prevents sizing with CSS, so we have to set
//         // the size manually when on web.
//         use winit::dpi::PhysicalSize;
//         let _ = window.request_inner_size(PhysicalSize::new(450, 400));

//         use winit::platform::web::WindowExtWebSys;
//         web_sys::window()
//             .and_then(|win| win.document())
//             .and_then(|doc| {
//                 let dst = doc.get_element_by_id("wasm")?;
//                 let canvas = web_sys::Element::from(window.canvas()?);
//                 dst.append_child(&canvas).ok()?;
//                 Some(())
//             })
//             .expect("Couldn't append canvas to document body.");
//     }
//     let mut state: State<'_> = state::State::new(&window).await;
//     let mut input_handler = input::InputHandler::new();

//     let textures = state.textures();

//     let render_system = render_system::RenderSystem::new(
//         &textures,
//         &state.context,
//         &state.world_uniform,
//         &state.camera,
//     );

//     let bg1 = {
//         let position_component = component::PositionComponent {
//             position: cgmath::Vector2::new(
//                 uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32 / 2.0,
//                 uniform::WorldUniform::WORLD_SCREEN_HEIGHT as f32 / 2.0,
//             ),
//             scale: cgmath::Vector2::new(
//                 uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32,
//                 uniform::WorldUniform::WORLD_SCREEN_HEIGHT as f32,
//             ),
//             is_controllable: false,
//         };
//         let vertex_array_component: component::VertexArrayComponent =
//             component::VertexArrayComponent::textured_quad(
//                 2,
//                 component::VertexArrayComponent::BACKGROUND_Z,
//             );

//         state.add_entity(
//             Some(position_component),
//             Some(vertex_array_component),
//             None,
//             None,
//             None,
//             None,
//             None,
//         )
//     };

//     let ground = {
//         let position_component = component::PositionComponent {
//             position: cgmath::Vector2::new(
//                 uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32 / 2.0,
//                 50.,
//             ),
//             scale: cgmath::Vector2::new(uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32, 100.),
//             is_controllable: false,
//         };

//         let vertex_array_component: component::VertexArrayComponent =
//             component::VertexArrayComponent::textured_quad(
//                 999,
//                 component::VertexArrayComponent::FOREGROUND_Z,
//             );

//         let collider_box_component = ColliderBoxComponent {
//             bottom_left: position_component.position - position_component.scale / 2.0,
//             top_right: position_component.position + position_component.scale / 2.0,
//         };

//         state.add_entity(
//             Some(position_component),
//             Some(vertex_array_component),
//             None,
//             None,
//             None,
//             Some(collider_box_component),
//             None,
//         )
//     };

//     let light = {
//         let position_component = component::PositionComponent {
//             position: cgmath::Vector2::new(100., 200.),
//             scale: cgmath::Vector2::new(30., 30.),
//             is_controllable: false,
//         };

//         let vertex_array_component: component::VertexArrayComponent =
//             component::VertexArrayComponent::circle(component::VertexArrayComponent::FOREGROUND_Z);

//         let light_component = uniform::LightComponent {
//             linear_dropoff: 0.001,
//             quadratic_dropoff: 0.0001,
//             ambient_strength: 3.,
//             diffuse_strength: 5.,
//             color: cgmath::Vector3 {
//                 x: 1.0,
//                 y: 0.0,
//                 z: 0.0,
//             },
//         };

//         state.add_entity(
//             Some(position_component),
//             Some(vertex_array_component),
//             None,
//             None,
//             None,
//             None,
//             Some(light_component),
//         )
//     };

//     let light2 = {
//         let position_component = component::PositionComponent {
//             position: cgmath::Vector2::new(500., 200.),
//             scale: cgmath::Vector2::new(30., 30.),
//             is_controllable: false,
//         };

//         let vertex_array_component: component::VertexArrayComponent =
//             component::VertexArrayComponent::circle(component::VertexArrayComponent::FOREGROUND_Z);

//         let light_component = uniform::LightComponent {
//             linear_dropoff: 0.001,
//             quadratic_dropoff: 0.0001,
//             ambient_strength: 3.,
//             diffuse_strength: 5.,
//             color: cgmath::Vector3 {
//                 x: 1.0,
//                 y: 1.0,
//                 z: 0.0,
//             },
//         };

//         state.add_entity(
//             Some(position_component),
//             Some(vertex_array_component),
//             None,
//             None,
//             None,
//             None,
//             Some(light_component),
//         )
//     };

//     // entity for player
//     let character = {
//         let position_component = component::PositionComponent {
//             position: cgmath::Vector2::new(82., 132.),
//             scale: cgmath::Vector2::new(64., 64.),
//             is_controllable: true,
//         };

//         let texture_index = 0; // warrior

//         let vertex_array_component = component::VertexArrayComponent::textured_quad(
//             texture_index,
//             component::VertexArrayComponent::OBJECT_Z,
//         );

//         let sprite_animation_idle = animation::SpriteAnimation {
//             animation_index: 0,
//             sprite_count: 10,
//             start_index: 0,
//             per_sprite_duration: Duration::new(0, 125000000),
//             current_elapsed_time: Duration::new(0, 0),
//         };
//         let sprite_animation_run = animation::SpriteAnimation {
//             animation_index: 0,
//             sprite_count: 10,
//             start_index: 20,
//             per_sprite_duration: Duration::new(0, 125000000),
//             current_elapsed_time: Duration::new(0, 0),
//         };
//         let sprite_animation_attack = animation::SpriteAnimation {
//             animation_index: 0,
//             sprite_count: 10,
//             start_index: 30,
//             per_sprite_duration: Duration::new(0, 125000000),
//             current_elapsed_time: Duration::new(0, 0),
//         };

//         let mut sprite_animation_controller = animation::SpriteAnimationControllerComponent::new();
//         sprite_animation_controller
//             .animation_map
//             .insert(component::CharacterState::IDLE, sprite_animation_idle);
//         sprite_animation_controller
//             .animation_map
//             .insert(component::CharacterState::MOVE, sprite_animation_run);
//         sprite_animation_controller
//             .animation_map
//             .insert(component::CharacterState::ATTACK, sprite_animation_attack);

//         let sheet_position_component = sprite::SheetPositionComponent {
//             sprite_sheet: state.sprite_sheets[texture_index as usize].clone(),
//             sheet_position: cgmath::Vector2::new(0, 0),
//         };

//         let character_state_component = component::CharacterStateComponent {
//             character_state: component::CharacterState::IDLE,
//         };

//         let collider_box_component = ColliderBoxComponent {
//             bottom_left: position_component.position - position_component.scale / 2.0,
//             top_right: position_component.position + position_component.scale / 2.0,
//         };

//         state.add_entity(
//             Some(position_component),
//             Some(vertex_array_component),
//             Some(sprite_animation_controller),
//             Some(sheet_position_component),
//             Some(character_state_component),
//             Some(collider_box_component),
//             None,
//         )
//     };

//     let minotaur = {
//         let position_component = component::PositionComponent {
//             position: cgmath::Vector2::new(232., 132.),
//             scale: cgmath::Vector2::new(64., 64.),
//             is_controllable: false,
//         };

//         let texture_index = 1; // warrior

//         let vertex_array_component = component::VertexArrayComponent::textured_quad(
//             texture_index,
//             component::VertexArrayComponent::OBJECT_Z,
//         );
//         let sprite_animation_idle = animation::SpriteAnimation {
//             animation_index: 0,
//             sprite_count: 10,
//             start_index: 0,
//             per_sprite_duration: Duration::new(0, 125000000),
//             current_elapsed_time: Duration::new(0, 0),
//         };
//         let sprite_animation_run = animation::SpriteAnimation {
//             animation_index: 0,
//             sprite_count: 10,
//             start_index: 20,
//             per_sprite_duration: Duration::new(0, 125000000),
//             current_elapsed_time: Duration::new(0, 0),
//         };
//         let sprite_animation_attack = animation::SpriteAnimation {
//             animation_index: 0,
//             sprite_count: 10,
//             start_index: 30,
//             per_sprite_duration: Duration::new(0, 125000000),
//             current_elapsed_time: Duration::new(0, 0),
//         };

//         let mut sprite_animation_controller = animation::SpriteAnimationControllerComponent::new();
//         sprite_animation_controller
//             .animation_map
//             .insert(component::CharacterState::IDLE, sprite_animation_idle);
//         sprite_animation_controller
//             .animation_map
//             .insert(component::CharacterState::MOVE, sprite_animation_run);
//         sprite_animation_controller
//             .animation_map
//             .insert(component::CharacterState::ATTACK, sprite_animation_attack);

//         let sheet_position_component = sprite::SheetPositionComponent {
//             sprite_sheet: state.sprite_sheets[texture_index as usize].clone(),
//             sheet_position: cgmath::Vector2::new(0, 0),
//         };

//         let character_state_component = component::CharacterStateComponent {
//             character_state: component::CharacterState::IDLE,
//         };

//         let collider_box_component = ColliderBoxComponent {
//             bottom_left: position_component.position - position_component.scale / 2.0,
//             top_right: position_component.position + position_component.scale / 2.0,
//         };

//         state.add_entity(
//             Some(position_component),
//             Some(vertex_array_component),
//             Some(sprite_animation_controller),
//             Some(sheet_position_component),
//             Some(character_state_component),
//             Some(collider_box_component),
//             None,
//         )
//     };

//     debug!("{:?}", state.vertex_array_components);
//     debug!(
//         "{:?}",
//         // state.camera.get_matrix() *
//         state
//             .world_uniform
//             .calc(state.size.width, state.size.height)
//             * cgmath::vec4(100., 300., 0.5, 1.)
//     );

//     let start_time = Instant::now();
//     let mut frames = 0;
//     let mut seconds_elapsed: u64 = 0;
//     let mut last_frame_time: Duration = Duration::new(0, 0);

//     const FIXED_UPDATES_PER_SECOND: u32 = 50;
//     const FIXED_UPDATE_DURATION: Duration = Duration::new(0, 1000000000 / FIXED_UPDATES_PER_SECOND);
//     let mut ticks_elapsed = Duration::new(0, 0);

//     let physics_system = physics::PhysicsSystem::new(FIXED_UPDATE_DURATION);

//     let _ = event_loop.run(move |event, control_flow| {
//         {
//             frames += 1;
//             let current_time = start_time.elapsed();
//             let delta_time = current_time - last_frame_time;
//             if current_time > Duration::new(seconds_elapsed + 1, 0) {
//                 debug!("FPS {:?}", frames);
//                 frames = 0;
//                 seconds_elapsed += 1;
//             }
//             last_frame_time = current_time;

//             ticks_elapsed += delta_time;
//             while ticks_elapsed > FIXED_UPDATE_DURATION {
//                 physics_system.update(
//                     &input_handler,
//                     &mut state.position_components,
//                     &mut state.collider_box_components,
//                 );

//                 ticks_elapsed -= FIXED_UPDATE_DURATION;
//             }

//             sprite::SpriteSheetSystem::update(
//                 &mut state.vertex_array_components,
//                 &state.sheet_position_components,
//             );
//             animation::AnimationSystem::update_animations(
//                 &mut state.sprite_animation_controller_components,
//                 &mut state.sheet_position_components,
//                 &state.character_state_components,
//                 delta_time,
//             );

//             // idle_anim.update(delta_time);
//             // state
//             //     .sprite
//             //     .update_sheet_position(idle_anim.get_sheet_index());
//             match event {
//                 Event::WindowEvent {
//                     ref event,
//                     window_id,
//                 } if window_id == state.window.id() => {
//                     match event {
//                         WindowEvent::CloseRequested
//                         | WindowEvent::KeyboardInput {
//                             event:
//                                 KeyEvent {
//                                     state: ElementState::Pressed,
//                                     physical_key: PhysicalKey::Code(KeyCode::Escape),
//                                     ..
//                                 },
//                             ..
//                         } => control_flow.exit(),
//                         WindowEvent::Resized(physical_size) => state.resize(*physical_size),

//                         WindowEvent::RedrawRequested => {
//                             let render_result = render_system.render(
//                                 &state.position_components,
//                                 &state.vertex_array_components,
//                                 &state.light_components,
//                                 &textures,
//                                 &state.context,
//                                 &state.gui,
//                                 true,
//                                 current_time,
//                             );
//                             match render_result {
//                                 Ok(_) => {}
//                                 // Reconfigure the surface if lost
//                                 Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
//                                     state.resize(state.size)
//                                 }
//                                 // The system is out of memory, we should probably quit
//                                 Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
//                                 // All other errors (Outdated, Timeout) should be resolved by the next frame
//                                 Err(e) => eprintln!("{:?}", e),
//                             }
//                         }

//                         WindowEvent::CursorMoved {
//                             device_id: _,
//                             position,
//                         } => input_handler.set_position(*position),
//                         WindowEvent::KeyboardInput {
//                             device_id: _,
//                             event,
//                             is_synthetic: _,
//                         } => input_handler.handle_key_state(
//                             event,
//                             &mut state.position_components,
//                             &mut state.character_state_components,
//                             &mut state.vertex_array_components,
//                         ),

//                         _ => {}
//                     }
//                 }

//                 Event::AboutToWait => {
//                     // RedrawRequested will only trigger once unless we manually
//                     // request it.
//                     state.window().request_redraw();
//                 }
//                 _ => {}
//             }
//         };
//     });
// }
