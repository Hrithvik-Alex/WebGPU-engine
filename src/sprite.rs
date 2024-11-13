use std::sync::Arc;

use crate::context;
use crate::model;
use crate::texture;

use image::GenericImageView;
use image::ImageBuffer;
pub struct Sprite {
    sprite_sheet: Arc<SpriteSheet>,
    vertices: [model::ModelVertex; 4],
    indices: [u16; 6],
}

impl Sprite {
    pub fn new(sprite_sheet: &Arc<SpriteSheet>) -> Self {
        let vertices: [model::ModelVertex; 4] = [
            // Changed
            model::ModelVertex {
                position: [0.0, 360.0, 1.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 0.0],
            }, // A
            model::ModelVertex {
                position: [640.0, 360.0, 1.0],
                tex_coords: [
                    sprite_sheet.sprite_width as f32 / sprite_sheet.dimensions.0 as f32,
                    0.0,
                ],
                normal: [0.0, 0.0, 0.0],
            }, // B
            model::ModelVertex {
                position: [0.0, 0.0, 1.0],
                tex_coords: [
                    0.0,
                    sprite_sheet.sprite_height as f32 / sprite_sheet.dimensions.1 as f32,
                ],
                normal: [0.0, 0.0, 0.0],
            }, // C
            model::ModelVertex {
                position: [640.0, 0.0, 1.0],
                tex_coords: [
                    (sprite_sheet.sprite_width as f32) / sprite_sheet.dimensions.0 as f32,
                    sprite_sheet.sprite_height as f32 / sprite_sheet.dimensions.1 as f32,
                ],
                normal: [0.0, 0.0, 0.0],
            }, // D
        ];

        let indices: [u16; 6] = [0, 2, 3, 0, 3, 1];
        Self {
            sprite_sheet: sprite_sheet.clone(),
            vertices,
            indices,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.sprite_sheet.bind_group
    }

    pub fn vertices(&self) -> &[model::ModelVertex; 4] {
        &self.vertices
    }

    pub fn indices(&self) -> &[u16; 6] {
        &self.indices
    }

    pub fn render(&self) {}
}

pub struct SpriteSheet {
    image_path: String,
    sprite_width: u32,
    sprite_height: u32,
    num_sprites: u32,
    dimensions: (u32, u32),
    texture: texture::Texture,
    bind_group: wgpu::BindGroup,
}

impl SpriteSheet {
    pub fn new(
        context: &context::Context,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
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

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    },
                ],
                label: Some("diffuse_bind_group"),
            });

        Self {
            image_path,
            sprite_width,
            sprite_height,
            num_sprites,
            dimensions,
            texture,
            bind_group,
        }
    }
}
