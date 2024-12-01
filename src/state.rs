use crate::animation;
use crate::camera;
use crate::component;
use crate::component::EntityMap;
use crate::context;
use crate::sprite;
use crate::texture;

use std::sync::Arc;

use winit::window::Window;

pub struct State<'a> {
    pub context: context::Context<'a>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: &'a Window,
    // TODO: decouple sprite sheet and textures
    pub sprite_sheets: Vec<Arc<sprite::SpriteSheet>>,
    pub position_components: component::EntityMap<component::PositionComponent>,
    pub camera: camera::OrthographicCamera,
    pub world_uniform: component::WorldUniform,
    pub vertex_array_components: component::EntityMap<component::VertexArrayComponent>, // camera: camera::Camera,
    pub sprite_animation_components: component::EntityMap<animation::SpriteAnimation>,
    pub sheet_position_components: component::EntityMap<sprite::SheetPositionComponent>,
    // entities: Vec<component::Entity>,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code

    pub async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();
        let context: context::Context<'a> = context::Context::new(window).await;

        let hero_sprite_sheet = Arc::new(sprite::SpriteSheet::new(
            &context,
            "./assets/warrior_spritesheet_calciumtrice.png".to_string(),
            Some("./assets/minotaur_spritesheet_calciumtrice_n.png".to_string()),
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

        let sprite_sheets = vec![hero_sprite_sheet.clone(), minotaur_sprite_sheet.clone()];

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

        let mut world_uniform = component::WorldUniform::new();
        world_uniform.resize(size.width, size.height);

        let position_components = EntityMap::new();
        let vertex_array_components = EntityMap::new();
        let sprite_animation_components = EntityMap::new();
        let sheet_position_components = EntityMap::new();

        // let entities = position_components
        //     .keys()
        //     .collect::<Vec<component::Entity>>();

        Self {
            window,
            context,
            size,
            position_components,
            sprite_sheets,
            camera,
            world_uniform,
            vertex_array_components,
            sprite_animation_components,
            sheet_position_components,
            // entities,
        }
    }

    pub fn add_entity(
        &mut self,
        position_component: Option<component::PositionComponent>,
        vertex_array_component: Option<component::VertexArrayComponent>,
        sprite_animation_component: Option<animation::SpriteAnimation>,
        sheet_position_component: Option<sprite::SheetPositionComponent>,
    ) -> component::Entity {
        let entity = self.position_components.insert(position_component);
        self.vertex_array_components.insert(vertex_array_component);

        self.sprite_animation_components
            .insert(sprite_animation_component);

        self.sheet_position_components
            .insert(sheet_position_component);

        // self.entities.push(entity);

        entity
    }

    pub fn remove_entity(&mut self, entity: component::Entity) {
        self.position_components.remove(entity);
        self.vertex_array_components.remove(entity);
        self.sprite_animation_components.remove(entity);
        self.sheet_position_components.remove(entity);
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
