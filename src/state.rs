use crate::animation;
use crate::camera;
use crate::component;
use crate::component::EntityMap;
use crate::context;
use crate::model;
use crate::render_system;
use crate::render_system::RenderSystem;
use crate::sprite;
use crate::texture;

use std::process::id;
use std::sync::Arc;
use std::time::Duration;

use model::Vertex;

use log::debug;
use winit::{dpi::PhysicalPosition, event::*};

use winit::window::Window;

pub struct State<'a> {
    pub context: context::Context<'a>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: &'a Window,
    // position: PhysicalPosition<f64>,
    // render_pipeline: wgpu::RenderPipeline,
    // index_buffer: wgpu::Buffer,
    // num_indices: u32,
    // TODO: decouple sprite sheet and textures
    sprite_sheets: Vec<Arc<sprite::SpriteSheet>>,
    // sprite_sheet: Arc<sprite::SpriteSheet>,
    pub position_components: component::EntityMap<component::PositionComponent>,
    render_system: render_system::RenderSystem,
    sprite: sprite::Sprite,
    pub camera: camera::OrthographicCamera,
    pub world_uniform: component::WorldUniform,
    pub vertex_array_components: component::EntityMap<component::VertexArrayComponent>, // camera: camera::Camera,
    pub sprite_animation_components: component::EntityMap<animation::SpriteAnimation>,
    pub sheet_position_components: component::EntityMap<sprite::SheetPositionComponent>,
    // projection: camera::Projection,
    // uniform_bind_group: wgpu::BindGroup,
    entities: Vec<component::Entity>,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code

    pub async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();
        let context: context::Context<'a> = context::Context::new(window).await;

        let hero_sprite_sheet = Arc::new(sprite::SpriteSheet::new(
            &context,
            "./assets/warrior_spritesheet_calciumtrice.png".to_string(),
            32,
            32,
            true,
        ));

        let sprite_sheets = vec![hero_sprite_sheet.clone()];

        let textures = sprite_sheets
            .iter()
            .map(|sprite_sheet| sprite_sheet.texture())
            .collect::<Vec<Arc<texture::Texture>>>();

        let camera = camera::OrthographicCamera::new(
            size.width,
            size.height,
            0.1,
            100.0,
            cgmath::Vector3::new(size.width as f32 / 2.0, size.height as f32 / 2.0, 1.0),
        );

        let mut world_uniform = component::WorldUniform::new();
        world_uniform.resize(size.width, size.height);

        let sprite_position_comp = component::PositionComponent {
            position: cgmath::Vector2::new(50., 100.),
            scale: 64.,
        };
        let sprite = sprite::Sprite::new(
            hero_sprite_sheet.clone(),
            sprite_position_comp.scale,
            sprite_position_comp.position,
        );

        let mut position_components = EntityMap::new();
        position_components.insert(sprite_position_comp);

        let quad = component::VertexArrayComponent::quad();

        let mut vertex_array_components = EntityMap::new();
        vertex_array_components.insert(quad);

        let idle_anim = animation::SpriteAnimation {
            animation_index: 0,
            sprite_count: 10,
            start_index: 0,
            per_sprite_duration: Duration::new(0, 125000000),
            current_elapsed_time: Duration::new(0, 0),
        };
        let mut sprite_animation_components = EntityMap::new();
        sprite_animation_components.insert(idle_anim);

        let sheet_position_comp = sprite::SheetPositionComponent {
            sprite_sheet: hero_sprite_sheet.clone(),
            sheet_position: cgmath::Vector2::new(0, 0),
        };

        let mut sheet_position_components = EntityMap::new();
        sheet_position_components.insert(sheet_position_comp);
        // hero_sprite_sheet.adjust_tex_coords(&mut quad, sprite.sheet_position);

        // let camera = camera::Camera::new(cgmath::Vector3::new(0.0, 0.0, 5.0));
        // let camera_buffer = camera.get_buffer(&context.device);

        // let projection =
        //     camera::Projection::new(size.width, size.height, cgmath::Deg(45.0), 0.1, 100.0, true);
        // debug!("proj:{:?}", (projection.calc_matrix()));
        debug!("bruh:{:?}", size);

        // let projection_buffer = projection.get_buffer(&context.device);
        let render_system =
            render_system::RenderSystem::new(textures, &context, &world_uniform, &camera);

        let entities = position_components
            .keys()
            .collect::<Vec<component::Entity>>();

        Self {
            window,
            context,
            size,
            // position: PhysicalPosition { x: 0.0, y: 0.0 },
            // render_pipeline,
            // index_buffer,
            // num_indices,
            render_system,
            position_components,
            sprite_sheets,
            sprite,
            camera,
            world_uniform,
            vertex_array_components, // projection,
            sprite_animation_components,
            sheet_position_components,
            // uniform_bind_group,
            entities,
        }
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

    pub fn set_position(&mut self, position: PhysicalPosition<f64>) {
        // self.position = position
    }
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let textures = self
            .sprite_sheets
            .iter()
            .map(|sprite_sheet| sprite_sheet.texture())
            .collect::<Vec<Arc<texture::Texture>>>();

        // self.sprite_sheets[0].adjust_tex_coords(&mut self.quad, self.sprite.sheet_position);
        self.render_system.render(
            &self.position_components,
            &self.vertex_array_components,
            textures,
            &self.context,
        )
        //             label: Some("Vertex Buffer"),
        //             contents: bytemuck::cast_slice(&self.sprite.vertices()),
        //             usage: wgpu::BufferUsages::VERTEX,
        //         });

        // let output = self.context.surface.get_current_texture()?;
        // let view = output
        //     .texture
        //     .create_view(&wgpu::TextureViewDescriptor::default());
        // let mut encoder =
        //     self.context
        //         .device
        //         .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        //             label: Some("Render Encoder"),
        //         });

        // {
        //     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //         label: Some("Render Pass"),
        //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //             view: &view,
        //             resolve_target: None,
        //             ops: wgpu::Operations {
        //                 load: wgpu::LoadOp::Clear(wgpu::Color {
        //                     r: self.position.x / f64::from(self.size.width),
        //                     g: self.position.y / f64::from(self.size.height),
        //                     b: 0.7,
        //                     a: 1.0,
        //                 }),
        //                 store: wgpu::StoreOp::Store,
        //             },
        //         })],
        //         depth_stencil_attachment: None,
        //         occlusion_query_set: None,
        //         timestamp_writes: None,
        //     });

        //     render_pass.set_pipeline(&self.render_pipeline);
        //     render_pass.set_bind_group(0, self.sprite.bind_group(), &[]);
        //     render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
        //     render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        //     render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
        //     render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        // }

        // self.context.queue.submit(std::iter::once(encoder.finish()));
        // output.present();

        // Ok(())
    }
}
