use anyhow::*;
use image::GenericImageView;

// TODO: refactor/consolidate?
pub struct TextureBasic {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl TextureBasic {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24PlusStencil8; // 1.

    pub fn default_pixel_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: None,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        })
    }

    pub fn create_basic(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format.add_srgb_suffix(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = Self::default_pixel_sampler(&device);

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = Self::default_pixel_sampler(&device);

        Self {
            texture,
            view,
            sampler,
        }
    }
}

pub struct Texture {
    // #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub normal_info: Option<(wgpu::Texture, wgpu::TextureView)>,
    // pub view: wgpu::TextureView,
    // pub sampler: wgpu::Sampler,
    pub dimensions: (u32, u32),
    // pub bind_group_layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl Texture {
    pub fn from_path(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_path: String,
        normal_path: Option<String>,
        manual_premultiply: bool,
    ) -> Result<Self> {
        let texture_bytes =
            std::fs::read(texture_path.clone()).expect("Failed to read sprite sheet image");
        let normal_bytes = normal_path.map(|normal_path| {
            std::fs::read(normal_path.clone()).expect("Failed to read normal sprite sheet image")
        });

        Self::from_bytes(
            device,
            queue,
            &texture_bytes,
            normal_bytes.as_deref(),
            &texture_path,
            manual_premultiply,
        )
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bytes: &[u8],
        normal_bytes: Option<&[u8]>,
        label: &str,
        manual_premultiply: bool,
    ) -> Result<Self> {
        let texture_img = image::load_from_memory(texture_bytes)?;
        let normal_img = normal_bytes
            .map(|normal_bytes| image::load_from_memory(normal_bytes))
            .map(|result| result.ok())
            .flatten();
        Self::from_image(
            device,
            queue,
            &texture_img,
            normal_img.as_ref(),
            Some(label),
            manual_premultiply,
        )
    }

    fn get_texture_from_img(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        label: Option<&str>,
        manual_premultiply: bool,
        bind_group_layout_index: u32,
    ) -> (wgpu::Texture, wgpu::TextureView, (u32, u32)) {
        let rgba = img.to_rgba8();
        let rgba = if manual_premultiply {
            rgba.pixels()
                .map(|pixel| {
                    let mut rgba = pixel.0;
                    let alpha = rgba[3] as f32 / 255.0; // Normalize alpha value to [0, 1]

                    rgba[0] = (rgba[0] as f32 * alpha) as u8;
                    rgba[1] = (rgba[1] as f32 * alpha) as u8;
                    rgba[2] = (rgba[2] as f32 * alpha) as u8;
                    rgba
                })
                .flatten()
                .collect()
        } else {
            rgba.to_vec()
        };
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // TODO: potentially use config.format?
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            ..Default::default()
        });

        (texture, view, dimensions)
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_img: &image::DynamicImage,
        normal_img: Option<&image::DynamicImage>,
        label: Option<&str>,
        manual_premultiply: bool,
    ) -> Result<Self> {
        let (texture, texture_view, dimensions) =
            Self::get_texture_from_img(device, queue, texture_img, label, manual_premultiply, 0);
        let (normal_info, normal_dimensions) = match normal_img {
            Some(normal_img) => {
                let (t, v, d) = Self::get_texture_from_img(
                    device,
                    queue,
                    normal_img,
                    label,
                    manual_premultiply,
                    1,
                );
                (Some((t, v)), Some(d))
            }
            None => (None, None),
        };
        normal_dimensions.map(|normal_dimensions| {
            if dimensions != normal_dimensions {
                assert!(dimensions == normal_dimensions)
            }
        });

        // let views = vec![Some(texture_view), normal_view]
        //     .into_iter()
        //     .filter_map(|x| x)
        //     .collect::<Vec<wgpu::TextureView>>();
        // let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        //     address_mode_u: wgpu::AddressMode::ClampToEdge,
        //     address_mode_v: wgpu::AddressMode::ClampToEdge,
        //     address_mode_w: wgpu::AddressMode::ClampToEdge,
        //     mag_filter: wgpu::FilterMode::Nearest,
        //     min_filter: wgpu::FilterMode::Nearest,
        //     mipmap_filter: wgpu::FilterMode::Nearest,
        //     ..Default::default()
        // });
        // let bind_group_layout_entries = vec![
        //     Some(texture_bind_group_layout),
        //     normal_bind_group_layout,
        //     Some(wgpu::BindGroupLayoutEntry {
        //         binding: if normal_bind_group_layout.is_some() {
        //             2
        //         } else {
        //             1
        //         },
        //         visibility: wgpu::ShaderStages::FRAGMENT,
        //         // This should match the filterable field of the
        //         // corresponding Texture entry above.
        //         ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        //         count: None,
        //     }),
        // ]
        // .iter()
        // .filter_map(|x| *x)
        // .collect::<Vec<BindGroupLayoutEntry>>();

        // let normal_bind_group_entry =
        //     normal_view
        //         .as_ref()
        //         .map(|normal_view| wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::TextureView(normal_view),
        //         });

        // let bind_group_entries = vec![
        //     Some(wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: wgpu::BindingResource::TextureView(&texture_view),
        //     }),
        //     normal_bind_group_entry,
        //     Some(wgpu::BindGroupEntry {
        //         binding: if normal_bind_group_layout.is_some() {
        //             2
        //         } else {
        //             1
        //         },
        //         resource: wgpu::BindingResource::Sampler(&sampler),
        //     }),
        // ]
        // .into_iter()
        // .flatten()
        // .collect::<Vec<wgpu::BindGroupEntry<'_>>>();

        Ok(Self {
            texture,
            view: texture_view,
            normal_info,
            dimensions,
            // bind_group_entries,
            // bind_group_layout_entries,
        })
    }
}
