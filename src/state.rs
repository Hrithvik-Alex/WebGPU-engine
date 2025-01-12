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

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use log::debug;
use winit::window::Window;

pub struct State<'a> {
    pub context: context::Context<'a>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,
    pub gui: gui::Gui,
    pub sprite_sheets: Vec<Rc<RefCell<sprite::SpriteSheet>>>,
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
    pub metadata_components: component::EntityMap<component::MetadataComponent>,
    pub physics_components: component::EntityMap<physics::PhysicsComponent>,
    pub parallax_components: component::EntityMap<component::ParallaxComponent>,
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

        let hero_sprite_sheet = Rc::new(RefCell::new(sprite::SpriteSheet::new(
            &context,
            "./assets/warrior_spritesheet_calciumtrice.png".to_string(),
            Some("./assets/warrior_spritesheet_calciumtrice_n.png".to_string()),
            32,
            32,
            true,
        )));

        let scroll_sprite_sheet = Rc::new(RefCell::new(sprite::SpriteSheet::new(
            &context,
            "./assets/scroll.png".to_string(),
            None,
            16,
            16,
            true,
        )));

        // let minotaur_sprite_sheet = Rc::new(RefCell::new(sprite::SpriteSheet::new(
        //     &context,
        //     "./assets/minotaur_spritesheet_calciumtrice.png".to_string(),
        //     Some("./assets/minotaur_spritesheet_calciumtrice_n.png".to_string()),
        //     48,
        //     48,
        //     true,
        // )));

        // let bg_sprite_sheet = Rc::new(RefCell::new(sprite::SpriteSheet::new(
        //     &context,
        //     "./assets/world_layer_1.png".to_string(),
        //     None,
        //     640,
        //     360,
        //     true,
        // )));

        let parallax_1_sprite_sheet = Rc::new(RefCell::new(sprite::SpriteSheet::new(
            &context,
            "./assets/bg/P1.png".to_string(),
            None,
            576,
            324,
            true,
        )));

        let parallax_2_sprite_sheet = Rc::new(RefCell::new(sprite::SpriteSheet::new(
            &context,
            "./assets/bg/P2.png".to_string(),
            None,
            576,
            324,
            true,
        )));

        let parallax_3_sprite_sheet = Rc::new(RefCell::new(sprite::SpriteSheet::new(
            &context,
            "./assets/bg/P3.png".to_string(),
            None,
            576,
            324,
            true,
        )));

        let parallax_4_sprite_sheet = Rc::new(RefCell::new(sprite::SpriteSheet::new(
            &context,
            "./assets/bg/P4.png".to_string(),
            None,
            576,
            324,
            true,
        )));

        let sprite_sheets = vec![
            hero_sprite_sheet.clone(),
            scroll_sprite_sheet.clone(),
            parallax_1_sprite_sheet.clone(),
            parallax_2_sprite_sheet.clone(),
            parallax_3_sprite_sheet.clone(),
            parallax_4_sprite_sheet.clone(),
        ];

        // let textures = sprite_sheets
        //     .iter()
        //     .map(|sprite_sheet| sprite_sheet.texture())
        //     .collect::<Vec<Arc<texture::Texture>>>();

        let camera = camera::OrthographicCamera::new(size.width, size.height, 0.1, 100.0);

        let mut world_uniform = uniform::WorldUniform::new();
        world_uniform.resize(size.width, size.height);

        let position_components = EntityMap::new();
        let vertex_array_components = EntityMap::new();
        let sprite_animation_controller_components = EntityMap::new();
        let sheet_position_components = EntityMap::new();
        let character_state_components = EntityMap::new();
        let collider_box_components = EntityMap::new();
        let light_components = EntityMap::new();
        let metadata_components = EntityMap::new();
        let physics_components = EntityMap::new();
        let parallax_components = EntityMap::new();

        // let entities = position_components
        //     .keys()
        //     .collect::<Vec<component::Entity>>();
        let input_handler = input::InputHandler::new();

        let textures = sprite_sheets
            .iter()
            .map(|sprite_sheet| sprite_sheet.borrow().texture())
            .collect::<Vec<Arc<texture::Texture>>>();

        let render_system = render_system::RenderSystem::new(&textures, &context);

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
            metadata_components,
            physics_components,
            parallax_components,
            input_handler,
            render_system,
            physics_system,
        }
    }

    pub fn init(&mut self) -> component::Entity {
        let parallax_scale = cgmath::Vector2 {
            x: 320. / 576.,
            y: 180. / 324.,
        };

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
            };
            let layer = 1;

            let vertex_array_component: component::VertexArrayComponent =
                component::VertexArrayComponent::textured_quad_with_coords(
                    2,
                    component::VertexArrayComponent::BACKGROUND_Z * layer as f32,
                    parallax_scale,
                );

            let metadata_component = component::MetadataComponent::new(false, false);

            let parallax_component = component::ParallaxComponent {
                move_speed: 30.,
                layer,
            };

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                None,
                None,
                None,
                None,
                None,
                Some(metadata_component),
                Some(parallax_component),
            )
        };

        let bg2 = {
            let position_component = component::PositionComponent {
                position: cgmath::Vector2::new(
                    uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32 / 2.0,
                    uniform::WorldUniform::WORLD_SCREEN_HEIGHT as f32 / 2.0,
                ),
                scale: cgmath::Vector2::new(
                    uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32,
                    uniform::WorldUniform::WORLD_SCREEN_HEIGHT as f32,
                ),
            };
            let layer = 2;

            let vertex_array_component: component::VertexArrayComponent =
                component::VertexArrayComponent::textured_quad_with_coords(
                    3,
                    component::VertexArrayComponent::BACKGROUND_Z * layer as f32,
                    parallax_scale,
                );

            let metadata_component = component::MetadataComponent::new(false, false);

            let parallax_component = component::ParallaxComponent {
                move_speed: 20.,
                layer,
            };

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                None,
                None,
                None,
                None,
                None,
                Some(metadata_component),
                Some(parallax_component),
            )
        };

        let bg3 = {
            let position_component = component::PositionComponent {
                position: cgmath::Vector2::new(
                    uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32 / 2.0,
                    uniform::WorldUniform::WORLD_SCREEN_HEIGHT as f32 / 2.0,
                ),
                scale: cgmath::Vector2::new(
                    uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32,
                    uniform::WorldUniform::WORLD_SCREEN_HEIGHT as f32,
                ),
            };
            let layer = 3;

            let vertex_array_component: component::VertexArrayComponent =
                component::VertexArrayComponent::textured_quad_with_coords(
                    4,
                    component::VertexArrayComponent::BACKGROUND_Z * layer as f32,
                    parallax_scale,
                );

            let metadata_component = component::MetadataComponent::new(false, false);

            let parallax_component = component::ParallaxComponent {
                move_speed: 10.,
                layer,
            };

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                None,
                None,
                None,
                None,
                None,
                Some(metadata_component),
                Some(parallax_component),
            )
        };

        let bg4 = {
            let position_component = component::PositionComponent {
                position: cgmath::Vector2::new(
                    uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32 / 2.0,
                    uniform::WorldUniform::WORLD_SCREEN_HEIGHT as f32 / 2.0,
                ),
                scale: cgmath::Vector2::new(
                    uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32,
                    uniform::WorldUniform::WORLD_SCREEN_HEIGHT as f32,
                ),
            };
            let layer = 4;

            let vertex_array_component: component::VertexArrayComponent =
                component::VertexArrayComponent::textured_quad_with_coords(
                    5,
                    component::VertexArrayComponent::BACKGROUND_Z * layer as f32,
                    parallax_scale,
                );

            let metadata_component = component::MetadataComponent::new(false, false);

            let parallax_component = component::ParallaxComponent {
                move_speed: 0.,
                layer,
            };

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                None,
                None,
                None,
                None,
                None,
                Some(metadata_component),
                Some(parallax_component),
            )
        };

        let ground = {
            let position_component = component::PositionComponent {
                position: cgmath::Vector2::new(
                    uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32 / 2.0,
                    50.,
                ),
                scale: cgmath::Vector2::new(uniform::WorldUniform::WORLD_SCREEN_WIDTH as f32, 100.),
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

            let metadata_component = component::MetadataComponent::new(false, false);

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                None,
                None,
                None,
                Some(collider_box_component),
                None,
                Some(metadata_component),
                None,
            )
        };

        let light = {
            let position_component = component::PositionComponent {
                position: cgmath::Vector2::new(100., 200.),
                scale: cgmath::Vector2::new(30., 30.),
            };

            let vertex_array_component: component::VertexArrayComponent =
                component::VertexArrayComponent::circle(
                    component::VertexArrayComponent::FOREGROUND_Z,
                );

            let light_component = uniform::LightComponent {
                linear_dropoff: 0.0007,
                quadratic_dropoff: 0.0001,
                ambient_strength: 10.,
                diffuse_strength: 15.,
                color: cgmath::Vector3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            };

            let metadata_component = component::MetadataComponent::new(false, false);

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                None,
                None,
                None,
                None,
                Some(light_component),
                Some(metadata_component),
                None,
            )
        };

        let light2 = {
            let position_component = component::PositionComponent {
                position: cgmath::Vector2::new(500., 200.),
                scale: cgmath::Vector2::new(30., 30.),
            };

            let vertex_array_component: component::VertexArrayComponent =
                component::VertexArrayComponent::circle(
                    component::VertexArrayComponent::FOREGROUND_Z,
                );

            let light_component = uniform::LightComponent {
                linear_dropoff: 0.0007,
                quadratic_dropoff: 0.0001,
                ambient_strength: 10.,
                diffuse_strength: 15.,
                color: cgmath::Vector3 {
                    x: 1.0,
                    y: 1.0,
                    z: 0.0,
                },
            };
            let metadata_component = component::MetadataComponent::new(false, false);

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                None,
                None,
                None,
                None,
                Some(light_component),
                Some(metadata_component),
                None,
            )
        };

        // entity for player
        let character = {
            let position_component = component::PositionComponent {
                position: cgmath::Vector2::new(82., 132.),
                scale: cgmath::Vector2::new(64., 64.),
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

            let metadata_component = component::MetadataComponent::new(true, true);

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                Some(sprite_animation_controller),
                Some(sheet_position_component),
                Some(character_state_component),
                Some(collider_box_component),
                None,
                Some(metadata_component),
                None,
            )
        };

        let scroll = {
            let position_component = component::PositionComponent {
                position: cgmath::Vector2::new(232., 132.),
                scale: cgmath::Vector2::new(64., 64.),
            };

            let texture_index = 1; // scroll

            let mut vertex_array_component = component::VertexArrayComponent::textured_quad(
                texture_index,
                component::VertexArrayComponent::OBJECT_Z,
            );
            vertex_array_component.shader_type = component::ShaderType::COLLECTIBLE;

            let metadata_component = component::MetadataComponent::new(false, false);

            self.add_entity(
                Some(position_component),
                Some(vertex_array_component),
                None,
                None,
                None,
                None,
                None,
                Some(metadata_component),
                None,
            )
        };

        // let minotaur = {
        //     let position_component = component::PositionComponent {
        //         position: cgmath::Vector2::new(232., 132.),
        //         scale: cgmath::Vector2::new(64., 64.),
        //     };

        //     let texture_index = 1; // warrior

        //     let vertex_array_component = component::VertexArrayComponent::textured_quad(
        //         texture_index,
        //         component::VertexArrayComponent::OBJECT_Z,
        //     );
        //     let sprite_animation_idle = animation::SpriteAnimation {
        //         animation_index: 0,
        //         sprite_count: 10,
        //         start_index: 0,
        //         per_sprite_duration: Duration::new(0, 125000000),
        //         current_elapsed_time: Duration::new(0, 0),
        //     };
        //     let sprite_animation_run = animation::SpriteAnimation {
        //         animation_index: 0,
        //         sprite_count: 10,
        //         start_index: 20,
        //         per_sprite_duration: Duration::new(0, 125000000),
        //         current_elapsed_time: Duration::new(0, 0),
        //     };
        //     let sprite_animation_attack = animation::SpriteAnimation {
        //         animation_index: 0,
        //         sprite_count: 10,
        //         start_index: 30,
        //         per_sprite_duration: Duration::new(0, 125000000),
        //         current_elapsed_time: Duration::new(0, 0),
        //     };

        //     let mut sprite_animation_controller =
        //         animation::SpriteAnimationControllerComponent::new();
        //     sprite_animation_controller
        //         .animation_map
        //         .insert(component::CharacterState::IDLE, sprite_animation_idle);
        //     sprite_animation_controller
        //         .animation_map
        //         .insert(component::CharacterState::MOVE, sprite_animation_run);
        //     sprite_animation_controller
        //         .animation_map
        //         .insert(component::CharacterState::ATTACK, sprite_animation_attack);

        //     let sheet_position_component = sprite::SheetPositionComponent {
        //         sprite_sheet: self.sprite_sheets[texture_index as usize].clone(),
        //         sheet_position: cgmath::Vector2::new(0, 0),
        //     };

        //     let character_state_component = component::CharacterStateComponent {
        //         character_state: component::CharacterState::IDLE,
        //     };

        //     let collider_box_component = ColliderBoxComponent {
        //         bottom_left: position_component.position - position_component.scale / 2.0,
        //         top_right: position_component.position + position_component.scale / 2.0,
        //     };

        //     let metadata_component = component::MetadataComponent::new(true, false);

        //     self.add_entity(
        //         Some(position_component),
        //         Some(vertex_array_component),
        //         Some(sprite_animation_controller),
        //         Some(sheet_position_component),
        //         Some(character_state_component),
        //         Some(collider_box_component),
        //         None,
        //         Some(metadata_component),
        //         None,
        //     )
        // };

        debug!("{:?}", self.vertex_array_components);
        // debug!(
        //     "{:?}",
        //     // self.camera.get_matrix() *
        //     self.world_uniform.calc(self.size.width, self.size.height)
        //         * cgmath::vec4(100., 300., 0.5, 1.)
        // );

        character
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
        metadata_component: Option<component::MetadataComponent>,
        parallax_component: Option<component::ParallaxComponent>,
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

        assert!(metadata_component.is_some());
        self.metadata_components.insert(metadata_component);
        self.physics_components
            .insert(Some(physics::PhysicsComponent::new()));
        // self.entities.push(entity);
        self.parallax_components.insert(parallax_component);

        entity
    }

    // pub fn add_physics_component_to_entity(
    //     &mut self,
    //     entity: component::Entity,
    //     physics_component: physics::PhysicsComponent,
    // ) {
    //     if let Some(component) = self.physics_components.get_mut(entity) {
    //         *component = Some(physics_component);
    //     } else {
    //         assert!(false);
    //     }
    // }

    pub fn remove_entity(&mut self, entity: component::Entity) {
        self.position_components.remove(entity);
        self.vertex_array_components.remove(entity);
        self.sprite_animation_controller_components.remove(entity);
        self.sheet_position_components.remove(entity);
        self.character_state_components.remove(entity);
        self.collider_box_components.remove(entity);
        self.light_components.remove(entity);
        self.metadata_components.remove(entity);
        self.physics_components.remove(entity);
        // self.entities.
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.context.resize(new_size);
            self.camera.resize(new_size.width, new_size.height);
            self.world_uniform.resize(new_size.width, new_size.height);

            self.sprite_sheets
                .iter_mut()
                .for_each(|sprite_sheet| sprite_sheet.borrow_mut().resize(&self.context));

            let textures = &self.textures();
            self.render_system.resize(textures, &self.context);
        }
    }

    pub fn textures(&self) -> Vec<Arc<texture::Texture>> {
        self.sprite_sheets
            .iter()
            .map(|sprite_sheet| sprite_sheet.borrow().texture())
            .collect::<Vec<Arc<texture::Texture>>>()
    }
}
