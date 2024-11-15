use crate::camera;
use crate::context;
use crate::model;
use crate::sprite;

use std::sync::Arc;

use cgmath::SquareMatrix;
use model::Vertex;

use log::debug;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalPosition, event::*};

use winit::window::Window;

#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WorldUniform {
    mat: [[f32; 4]; 4],
}

impl WorldUniform {
    const WORLD_SCREEN_WIDTH: u32 = 640;
    const WORLD_SCREEN_HEIGHT: u32 = 360;

    pub fn new() -> Self {
        Self {
            mat: cgmath::Matrix4::identity().into(),
        }
    }

    fn calc(&self, width: u32, height: u32) -> cgmath::Matrix4<f32> {
        #[cfg_attr(rustfmt, rustfmt_skip)]
         cgmath::Matrix4::new(
            width as f32/Self::WORLD_SCREEN_WIDTH as f32, 0., 0., 0.,
            0., height as f32/Self::WORLD_SCREEN_HEIGHT as f32, 0., 0.,
            0., 0., 1., 0.,
            0., 0., 0., 1.,
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.mat = self.calc(width, height).into();
    }

    pub fn get_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ortho buffer"),
            contents: bytemuck::cast_slice(&[self.mat]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}

pub struct State<'a> {
    context: context::Context<'a>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub window: &'a Window,
    position: PhysicalPosition<f64>,
    render_pipeline: wgpu::RenderPipeline,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    sprite_sheet: Arc<sprite::SpriteSheet>,
    pub sprite: sprite::Sprite,
    camera: camera::OrthographicCamera,
    world_uniform: WorldUniform,
    // camera: camera::Camera,
    // projection: camera::Projection,
    uniform_bind_group: wgpu::BindGroup,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();
        let context = context::Context::new(window).await;

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

        // println!("Current directory: {:?}", std::env::current_dir().unwrap());
        let sprite_sheet = Arc::new(sprite::SpriteSheet::new(
            &context,
            &texture_bind_group_layout,
            "./assets/warrior_spritesheet_calciumtrice.png".to_string(),
            32,
            32,
            true,
        ));
        let sprite = sprite::Sprite::new(&sprite_sheet, 64., cgmath::Vector2::new(50., 100.));
        // let camera = camera::Camera::new(cgmath::Vector3::new(0.0, 0.0, 5.0));
        // let camera_buffer = camera.get_buffer(&context.device);

        // let projection =
        //     camera::Projection::new(size.width, size.height, cgmath::Deg(45.0), 0.1, 100.0, true);
        // debug!("proj:{:?}", (projection.calc_matrix()));
        debug!("bruh:{:?}", size);

        // let projection_buffer = projection.get_buffer(&context.device);

        let camera = camera::OrthographicCamera::new(
            size.width,
            size.height,
            0.1,
            100.0,
            cgmath::Vector3::new(size.width as f32 / 2.0, size.height as f32 / 2.0, 1.0),
        );

        let camera_buffer = camera.get_buffer(&context.device);
        let mut world_uniform = WorldUniform::new();
        world_uniform.resize(size.width, size.height);
        let world_buffer = world_uniform.get_buffer(&context.device);

        let uniform_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("camera bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let uniform_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            camera_buffer.as_entire_buffer_binding(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(
                            world_buffer.as_entire_buffer_binding(),
                        ),
                    },
                ],
                label: Some("camera bind group"),
            });

        let shader: wgpu::ShaderModule =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
                });

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let create_render_pipeline = |layout: &str,
                                      vertex_entry_point: &str,
                                      fragment_entry_point: &str|
         -> wgpu::RenderPipeline {
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(layout),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: vertex_entry_point,
                        buffers: &[model::ModelVertex2d::desc()],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: fragment_entry_point,
                        targets: &[Some(wgpu::ColorTargetState {
                            // 4.
                            format: context.config.format,
                            blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None, // TODO: what does this mean?
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        //TODO: What is multisampling?
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                })
        };

        let render_pipeline = create_render_pipeline("Render Pipeline", "vs_main", "fs_main");

        debug!("{:?}", (bytemuck::cast_slice::<_, f32>(&sprite.vertices())));
        debug!("{:?}", (bytemuck::cast_slice::<_, u16>(&sprite.indices())));

        let index_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&sprite.indices()),
                usage: wgpu::BufferUsages::INDEX,
            });

        let num_indices = sprite.indices().len() as u32;

        Self {
            window,
            context,
            size,
            position: PhysicalPosition { x: 0.0, y: 0.0 },
            render_pipeline,
            index_buffer,
            num_indices,
            sprite_sheet,
            sprite,
            camera,
            world_uniform,
            // projection,
            uniform_bind_group,
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
        self.position = position
    }
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let vertex_buffer =
            self.context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&self.sprite.vertices()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let output = self.context.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.position.x / f64::from(self.size.width),
                            g: self.position.y / f64::from(self.size.height),
                            b: 0.7,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, self.sprite.bind_group(), &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
