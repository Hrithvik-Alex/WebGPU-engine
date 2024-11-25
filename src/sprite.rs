use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use crate::component;
use crate::context;
use crate::model;
use crate::texture;

use cgmath::Vector2;
use log::debug;

pub struct Sprite {
    sprite_sheet: Arc<SpriteSheet>,
    position: Vector2<f32>,
    pub sheet_position: Vector2<u32>,
    scale: f32,
}

impl Sprite {
    pub fn new(sprite_sheet: &Arc<SpriteSheet>, scale: f32, position: Vector2<f32>) -> Self {
        Self {
            sprite_sheet: sprite_sheet.clone(),
            position,
            sheet_position: Vector2::new(0, 0),
            scale,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.sprite_sheet.texture.bind_group
    }

    pub fn vertices(&self) -> [model::ModelVertex2d; 4] {
        let x = self.position.x;
        let y = self.position.y;

        let sheet_x = self.sheet_position.x as f32 * self.sprite_sheet.sprite_width as f32
            / self.sprite_sheet.dimensions.0 as f32;
        let sheet_y = self.sheet_position.y as f32 * self.sprite_sheet.sprite_height as f32
            / self.sprite_sheet.dimensions.1 as f32;

        [
            // Changed
            model::ModelVertex2d {
                // TOP-LEFT
                position: [x, y + self.scale],
                tex_coords: [sheet_x, sheet_y],
                normal: [0.0, 0.0, 0.0],
            },
            model::ModelVertex2d {
                // TOP-RIGHT
                position: [x + self.scale, y + self.scale],
                tex_coords: [
                    sheet_x
                        + self.sprite_sheet.sprite_width as f32
                            / self.sprite_sheet.dimensions.0 as f32,
                    sheet_y,
                ],
                normal: [0.0, 0.0, 0.0],
            },
            model::ModelVertex2d {
                // BOTTOM-LEFT
                position: [x, y],
                tex_coords: [
                    sheet_x,
                    sheet_y
                        + self.sprite_sheet.sprite_height as f32
                            / self.sprite_sheet.dimensions.1 as f32,
                ],
                normal: [0.0, 0.0, 0.0],
            },
            model::ModelVertex2d {
                // BOTTOM-RIGHT
                position: [x + self.scale, y],
                tex_coords: [
                    sheet_x
                        + (self.sprite_sheet.sprite_width as f32)
                            / self.sprite_sheet.dimensions.0 as f32,
                    sheet_y
                        + self.sprite_sheet.sprite_height as f32
                            / self.sprite_sheet.dimensions.1 as f32,
                ],
                normal: [0.0, 0.0, 0.0],
            },
        ]
    }

    pub fn indices(&self) -> [u16; 6] {
        [0, 2, 3, 0, 3, 1]
    }
    pub fn get_position(&self) -> Vector2<f32> {
        self.position
    }
    pub fn update_position(&mut self, position: Vector2<f32>) {
        self.position = position;
    }

    pub fn update_sheet_position(&mut self, sheet_index: u32) {
        let sheet_position = self.sprite_sheet.get_position_by_index(sheet_index);
        self.sheet_position = sheet_position;
    }
}

pub struct SpriteAnimation {
    pub animation_index: u32,
    pub sprite_count: u32,
    pub start_index: u32,
    pub per_sprite_duration: Duration,
    pub current_elapsed_time: Duration,
}

impl SpriteAnimation {
    pub fn update(&mut self, delta_time: Duration) {
        self.current_elapsed_time += delta_time;
        if self.current_elapsed_time > self.per_sprite_duration {
            self.current_elapsed_time -= self.per_sprite_duration;
            self.animation_index = (self.animation_index + 1) % self.sprite_count;
        }
    }

    pub fn get_sheet_index(&self) -> u32 {
        self.start_index + self.animation_index
    }
}

pub struct SpriteSheet {
    image_path: String,
    sprite_width: u32,
    sprite_height: u32,
    num_sprites: u32,
    dimensions: (u32, u32),
    pub texture: texture::Texture,
}

impl SpriteSheet {
    pub fn new(
        context: &context::Context,
        image_path: String,
        sprite_width: u32,
        sprite_height: u32,
        manual_premultiply: bool,
    ) -> Self {
        let bytes = std::fs::read(image_path.clone()).expect("Failed to read sprite sheet image");
        let texture = crate::texture::Texture::from_bytes(
            &context.device,
            &context.queue,
            &bytes,
            &image_path,
            manual_premultiply,
        )
        .unwrap();
        let dimensions = texture.dimensions;
        let num_sprites = (dimensions.0 / sprite_width) * (dimensions.1 / sprite_height);

        Self {
            image_path,
            sprite_width,
            sprite_height,
            num_sprites,
            dimensions,
            texture,
        }
    }

    pub fn get_position_by_index(&self, index: u32) -> Vector2<u32> {
        Vector2::new(index % self.dimensions.1, index / self.dimensions.1)
    }

    pub fn adjust_tex_coords(
        &self,
        vertex_array: &mut component::VertexArrayComponent,
        sheet_position: Vector2<u32>,
    ) {
        let step_x = self.sprite_width as f32 / self.dimensions.0 as f32;
        let step_y = self.sprite_height as f32 / self.dimensions.1 as f32;
        let sheet_x = sheet_position.x as f32 * step_x;
        let sheet_y = sheet_position.y as f32 * step_y;

        vertex_array.tex_coords = vertex_array
            .whole_tex_coords
            .iter()
            .map(|whole_tex_coord| {
                cgmath::Vector2::new(
                    sheet_x + whole_tex_coord.x * step_x,
                    sheet_y + whole_tex_coord.y * step_y,
                )
            })
            .collect();

        // debug!("{:?}", vertex_array.tex_coords);
        // debug!("{:?}", sheet_position);
    }
}
