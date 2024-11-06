pub struct Sprite {
    width: u32,
    height: u32,
    texture: texture::Texture,
}

impl Sprite {
    pub fn new(width: u32, height: u32, sprite_sheet: &SpriteSheet) -> Self {
        let bytes = include_bytes!(sprite_sheet.image);
        let texture = texture::Texture::from_bytes(
            &context.device,
            &context.queue,
            bytes,
            sprite_sheet.image,
        )
        .unwrap();

        let texture_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            // This should match the filterable field of the
                            // corresponding Texture entry above.
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("texture_bind_group_layout"),
                });
        const VERTICES: &[model::ModelVertex] = &[
            // Changed
            model::ModelVertex {
                position: [0.0, height, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 0.0],
            }, // A
            model::ModelVertex {
                position: [width, height, 0.0],
                tex_coords: [
                    sprite_sheet.sprite_width as f32 / sprite_sheet.dimensions.0 as f32,
                    0.0,
                ],
                normal: [0.0, 0.0, 0.0],
            }, // B
            model::ModelVertex {
                position: [0.0, 0.0, 0.0],
                tex_coords: [
                    0.0,
                    sprite_sheet.sprite_height as f32 / sprite_sheet.dimensions.1 as f32,
                ],
                normal: [0.0, 0.0, 0.0],
            }, // C
            model::ModelVertex {
                position: [width, 0.0, 0.0],
                tex_coords: [
                    sprite_sheet.sprite_width as f32 / sprite_sheet.dimensions.0 as f32,
                    sprite_sheet.sprite_height as f32 / sprite_sheet.dimensions.1 as f32,
                ],
                normal: [0.0, 0.0, 0.0],
            }, // D
        ];

        const INDICES: &[u16] = &[0, 3, 2, 0, 2, 1];
        Self {
            width,
            height,
            texture,
        }
    }

    pub fn render(&self) {}
}

pub struct SpriteSheet {
    image: String,
    sprite_width: u32,
    sprite_height: u32,
    num_sprites: u32,
    dimensions: (u32, u32),
}

impl SpriteSheet {
    pub fn new(image: String, sprite_width: u32, sprite_height: u32) -> Self {
        let img = image::open(image).unwrap();
        let dimensions = img.dimensions();
        let num_sprites = (dimensions.0 / sprite_width) * (dimensions.1 / sprite_height);

        Self {
            image,
            sprite_width,
            sprite_height,
            num_sprites,
            dimensions,
        }
    }

    pub fn get_sprite(&self, index: u32) -> Sprite {}
}
