use crate::animation;
use crate::camera;
use crate::component;
use crate::component::EntityMap;
use crate::context;
use crate::gui;
use crate::input;
use crate::physics;
use crate::physics::ColliderBoxComponent;
use crate::render_system;
use crate::sprite;
use crate::texture;
use crate::uniform;

use std::sync::Arc;
use std::time::{Duration, Instant};

use log::debug;
use winit::window::Window;

pub struct State<'a> {
    pub context: context::Context<'a>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,
    pub gui: gui::Gui,
    pub sprite_sheets: Vec<Arc<sprite::SpriteSheet>>,
    pub position_components: component::EntityMap<component::PositionComponent>,
    pub camera: camera::OrthographicCamera,
    pub world_uniform: uniform::WorldUniform,
    pub vertex_array_components: component::EntityMap<component::VertexArrayComponent>, // camera: camera::Camera,
    pub sprite_animation_controller_components:
        component::EntityMap<animation::SpriteAnimationControllerComponent>,
    pub sheet_position_components: component::EntityMap<sprite::SheetPositionComponent>,
    pub character_state_components: component::EntityMap<component::CharacterStateComponent>,
    pub collider_box_components: component::EntityMap<physics::ColliderBoxComponent>,
    pub light_components: component::EntityMap<uniform::LightComponent>,
    // entities: Vec<component::Entity>,

    // systems
    pub input_handler: input::InputHandler,
    pub render_system: render_system::RenderSystem,
    pub physics_system: physics::PhysicsSystem,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code

    pub const FIXED_UPDATES_PER_SECOND: u32 = 50;
    pub const FIXED_UPDATE_DURATION: Duration =
        Duration::new(0, 1000000000 / Self::FIXED_UPDATES_PER_SECOND);

    pub async fn new(window: Arc<Window>) -> State<'a> {
        let size = window.inner_size();
        let context: context::Context<'a> = context::Context::new(window.clone()).await;

        let gui = gui::Gui::new(
            &context.device,
            context.config.format,
            None,
            1,
            window.clone(),
        );

        let hero_sprite_sheet = Arc::new(sprite::SpriteSheet::new(
            &context,
            "./assets/warrior_spritesheet_calciumtrice.png".to_string(),
            Some("./assets/warrior_spritesheet_calciumtrice_n.png".to_string()),
            32,
            32,
            true,
        ));

        let minotaur_sprite_sheet = Arc::new(sprite::SpriteSheet::new(
            &context,
            "./assets/minotaur_spritesheet_calciumtrice.png".to_string(),
            Some("./assets/minotaur_spritesheet_calciumtrice_n.png".to_string()),
            48,
            48,
            true,
        ));

        let bg_sprite_sheet = Arc::new(sprite::SpriteSheet::new(
            &context,
            "./assets/world_layer_1.png".to_string(),
            None,
            640,
            360,
            true,
        ));

        let sprite_sheets = vec![
            hero_sprite_sheet.clone(),
            minotaur_sprite_sheet.clone(),
            bg_sprite_sheet.clone(),
        ];

        // let textures = sprite_sheets
        //     .iter()
        //     .map(|sprite_sheet| sprite_sheet.texture())
        //     .collect::<Vec<Arc<texture::Texture>>>();

        let camera = camera::OrthographicCamera::new(
            size.width,
            size.height,
            0.1,
            100.0,
            cgmath::Vector3::new(size.width as f32 / 2.0, size.height as f32 / 2.0, 1.0),
        );

        let mut world_uniform = uniform::WorldUniform::new();
        world_uniform.resize(size.width, size.height);

        let position_components = EntityMap::new();
        let vertex_array_components = EntityMap::new();
        let sprite_animation_controller_components = EntityMap::new();
        let sheet_position_components = EntityMap::new();
        let character_state_components = EntityMap::new();
        let collider_box_components = EntityMap::new();
        let light_components = EntityMap::new();

        // let entities = position_components
        //     .keys()
        //     .collect::<Vec<component::Entity>>();
        let input_handler = input::InputHandler::new();

        let textures = sprite_sheets
            .iter()
            .map(|sprite_sheet| sprite_sheet.texture())
            .collect::<Vec<Arc<texture::Texture>>>();

        let render_system =
            render_system::RenderSystem::new(&textures, &context, &world_uniform, &camera);

        let physics_system = physics::PhysicsSystem::new(Self::FIXED_UPDATE_DURATION);

        Self {
            window,
            context,
            size,
            position_components,
            sprite_sheets,
            camera,
            gui,
            world_uniform,
            vertex_array_components,
            sprite_animation_controller_components,
            sheet_position_components,
            character_state_components,
            collider_box_components, // entities,
            light_components,
            input_handler,
            render_system,
            physics_system,
        }
    }

    pub fn init(&mut self) {
        let bg1 = {
            let position_component = component::PositionComponent {
                position: cgmath::Vector2::new(
                    uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32 / 2.0,
                    uniform::WorldUniform::WORLD_SCREEN_HEIGHT as f32 / 2.0,
                ),
                scale: cgmath::Vector2::new(
                    uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32,
                    uniform::WorldUniform::WORLD_SCREEN_HEIGHT as f32,
                ),
                is_controllable: false,
            };
            let vertex_array_component: component::VertexArrayComponent =
                component::VertexArrayComponent::textured_quad(
                    2,
                    component::VertexArrayComponent::BACKGROUND_Z,
                );

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                None,
                None,
                None,
                None,
                None,
            )
        };

        let ground = {
            let position_component = component::PositionComponent {
                position: cgmath::Vector2::new(
                    uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32 / 2.0,
                    50.,
                ),
                scale: cgmath::Vector2::new(uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32, 100.),
                is_controllable: false,
            };

            let vertex_array_component: component::VertexArrayComponent =
                component::VertexArrayComponent::textured_quad(
                    999,
                    component::VertexArrayComponent::FOREGROUND_Z,
                );

            let collider_box_component = ColliderBoxComponent {
                bottom_left: position_component.position - position_component.scale / 2.0,
                top_right: position_component.position + position_component.scale / 2.0,
            };

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                None,
                None,
                None,
                Some(collider_box_component),
                None,
            )
        };

        let light = {
            let position_component = component::PositionComponent {
                position: cgmath::Vector2::new(100., 200.),
                scale: cgmath::Vector2::new(30., 30.),
                is_controllable: false,
            };

            let vertex_array_component: component::VertexArrayComponent =
                component::VertexArrayComponent::circle(
                    component::VertexArrayComponent::FOREGROUND_Z,
                );

            let light_component = uniform::LightComponent {
                linear_dropoff: 0.001,
                quadratic_dropoff: 0.0001,
                ambient_strength: 3.,
                diffuse_strength: 5.,
                color: cgmath::Vector3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            };

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                None,
                None,
                None,
                None,
                Some(light_component),
            )
        };

        let light2 = {
            let position_component = component::PositionComponent {
                position: cgmath::Vector2::new(500., 200.),
                scale: cgmath::Vector2::new(30., 30.),
                is_controllable: false,
            };

            let vertex_array_component: component::VertexArrayComponent =
                component::VertexArrayComponent::circle(
                    component::VertexArrayComponent::FOREGROUND_Z,
                );

            let light_component = uniform::LightComponent {
                linear_dropoff: 0.001,
                quadratic_dropoff: 0.0001,
                ambient_strength: 3.,
                diffuse_strength: 5.,
                color: cgmath::Vector3 {
                    x: 1.0,
                    y: 1.0,
                    z: 0.0,
                },
            };

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                None,
                None,
                None,
                None,
                Some(light_component),
            )
        };

        // entity for player
        let character = {
            let position_component = component::PositionComponent {
                position: cgmath::Vector2::new(82., 132.),
                scale: cgmath::Vector2::new(64., 64.),
                is_controllable: true,
            };

            let texture_index = 0; // warrior

            let vertex_array_component = component::VertexArrayComponent::textured_quad(
                texture_index,
                component::VertexArrayComponent::OBJECT_Z,
            );

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

            let mut sprite_animation_controller =
                animation::SpriteAnimationControllerComponent::new();
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
                sprite_sheet: self.sprite_sheets[texture_index as usize].clone(),
                sheet_position: cgmath::Vector2::new(0, 0),
            };

            let character_state_component = component::CharacterStateComponent {
                character_state: component::CharacterState::IDLE,
            };

            let collider_box_component = ColliderBoxComponent {
                bottom_left: position_component.position - position_component.scale / 2.0,
                top_right: position_component.position + position_component.scale / 2.0,
            };

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                Some(sprite_animation_controller),
                Some(sheet_position_component),
                Some(character_state_component),
                Some(collider_box_component),
                None,
            )
        };

        let minotaur = {
            let position_component = component::PositionComponent {
                position: cgmath::Vector2::new(232., 132.),
                scale: cgmath::Vector2::new(64., 64.),
                is_controllable: false,
            };

            let texture_index = 1; // warrior

            let vertex_array_component = component::VertexArrayComponent::textured_quad(
                texture_index,
                component::VertexArrayComponent::OBJECT_Z,
            );
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

            let mut sprite_animation_controller =
                animation::SpriteAnimationControllerComponent::new();
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
                sprite_sheet: self.sprite_sheets[texture_index as usize].clone(),
                sheet_position: cgmath::Vector2::new(0, 0),
            };

            let character_state_component = component::CharacterStateComponent {
                character_state: component::CharacterState::IDLE,
            };

            let collider_box_component = ColliderBoxComponent {
                bottom_left: position_component.position - position_component.scale / 2.0,
                top_right: position_component.position + position_component.scale / 2.0,
            };

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                Some(sprite_animation_controller),
                Some(sheet_position_component),
                Some(character_state_component),
                Some(collider_box_component),
                None,
            )
        };

        debug!("{:?}", self.vertex_array_components);
        debug!(
            "{:?}",
            // self.camera.get_matrix() *
            self.world_uniform.calc(self.size.width, self.size.height)
                * cgmath::vec4(100., 300., 0.5, 1.)
        );
    }

    pub fn add_entity(
        &mut self,
        position_component: Option<component::PositionComponent>,
        vertex_array_component: Option<component::VertexArrayComponent>,
        sprite_animation_controller_component: Option<
            animation::SpriteAnimationControllerComponent,
        >,
        sheet_position_component: Option<sprite::SheetPositionComponent>,
        character_state_component: Option<component::CharacterStateComponent>,
        collider_box_component: Option<physics::ColliderBoxComponent>,
        light_component: Option<uniform::LightComponent>,
    ) -> component::Entity {
        let entity = self.position_components.insert(position_component);
        self.vertex_array_components.insert(vertex_array_component);

        self.sprite_animation_controller_components
            .insert(sprite_animation_controller_component);

        self.sheet_position_components
            .insert(sheet_position_component);

        self.character_state_components
            .insert(character_state_component);

        self.collider_box_components.insert(collider_box_component);

        self.light_components.insert(light_component);

        // self.entities.push(entity);

        entity
    }

    pub fn remove_entity(&mut self, entity: component::Entity) {
        self.position_components.remove(entity);
        self.vertex_array_components.remove(entity);
        self.sprite_animation_controller_components.remove(entity);
        self.sheet_position_components.remove(entity);
        self.character_state_components.remove(entity);
        self.collider_box_components.remove(entity);
        self.light_components.remove(entity);
        // self.entities.
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.context.config.width = new_size.width;
            self.context.config.height = new_size.height;
            self.context
                .surface
                .configure(&self.context.device, &self.context.config);
            self.camera.resize(new_size.width, new_size.height);
            self.world_uniform.resize(new_size.width, new_size.height);
        }
    }

    pub fn textures(&self) -> Vec<Arc<texture::Texture>> {
        self.sprite_sheets
            .iter()
            .map(|sprite_sheet| sprite_sheet.texture())
            .collect::<Vec<Arc<texture::Texture>>>()
    }
}
