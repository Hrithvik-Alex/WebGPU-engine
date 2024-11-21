use std::iter;

use crate::camera;
use crate::component;
use crate::context;
use crate::model;
use crate::model::ModelVertex2d;
use crate::model::Vertex;
use crate::texture;

use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

pub struct RenderSystem<'a> {
    positions: Vec<&'a component::PositionComponent>,
    vertex_arrays: Vec<&'a component::VertexArrayComponent>,
    textures: Vec<&'a texture::Texture>,
    context: &'a context::Context<'a>,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
}

impl<'a> RenderSystem<'a> {
    pub fn new(
        positions: Vec<&'a component::PositionComponent>,
        vertex_arrays: Vec<&'a component::VertexArrayComponent>,
        textures: Vec<&'a texture::Texture>,
        context: &'a context::Context,
        size: PhysicalSize<u32>,
        world_uniform: &'a component::WorldUniform,
        camera: &'a camera::OrthographicCamera,
    ) -> Self {
        assert!(positions.len() == vertex_arrays.len());

        // let camera = camera::OrthographicCamera::new(
        //     size.width,
        //     size.height,
        //     0.1,
        //     100.0,
        //     cgmath::Vector3::new(size.width as f32 / 2.0, size.height as f32 / 2.0, 1.0),
        // );

        let camera_buffer = camera.get_buffer(&context.device);
        // let mut world_uniform = WorldUniform::new();
        // world_uniform.resize(size.width, size.height);
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

        let mut bind_group_layouts: Vec<&wgpu::BindGroupLayout> = textures
            .iter()
            .map(|&texture| &texture.bind_group_layout)
            .collect();

        bind_group_layouts.push(&uniform_bind_group_layout);

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &bind_group_layouts,
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

        // debug!("{:?}", (bytemuck::cast_slice::<_, f32>(&sprite.vertices())));
        // debug!("{:?}", (bytemuck::cast_slice::<_, u16>(&sprite.indices())));

        // let index_buffer = context
        //     .device
        //     .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //         label: Some("Index Buffer"),
        //         contents: bytemuck::cast_slice(&sprite.indices()),
        //         usage: wgpu::BufferUsages::INDEX,
        //     });

        // let num_indices = sprite.indices().len() as u32;

        Self {
            positions,
            vertex_arrays,
            textures,
            context,
            size,
            render_pipeline,
            uniform_bind_group,
        }
    }
    pub fn render(&self, context: &context::Context) -> Result<(), wgpu::SurfaceError> {
        // let model_vertices: Vec<model::ModelVertex2d> = self
        //     .positions
        //     .iter()
        //     .zip(self.vertex_arrays.iter())
        //     .map(|(&pos, &vertex_array)| {
        //         vertex_array
        //             .vertices
        //             .iter()
        //             .zip(vertex_array.tex_coords.iter())
        //             .map(|(vertex_pos, &tex_coord)| model::ModelVertex2d {
        //                 position: ((vertex_pos * pos.scale) + pos.position).into(),
        //                 tex_coords: tex_coord.into(),
        //                 normal: [0.0, 0.0, 0.0],
        //             })
        //     })
        //     .flatten()
        //     .collect();
        let mut all_vertices: Vec<ModelVertex2d> = vec![];
        let mut all_indices: Vec<u32> = vec![];
        for i in 0..self.positions.len() {
            let vertex_array = self.vertex_arrays[i];
            let pos = self.positions[i];
            let model_vertices = vertex_array
                .vertices
                .iter()
                .zip(vertex_array.tex_coords.iter())
                .map(|(vertex_pos, &tex_coord)| model::ModelVertex2d {
                    position: ((vertex_pos * pos.scale) + pos.position).into(),
                    tex_coords: tex_coord.into(),
                    normal: [0.0, 0.0, 0.0],
                });

            all_vertices.extend(model_vertices);
            all_indices.extend_from_slice(&vertex_array.indices);
        }

        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&all_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&all_indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let output = context.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = context
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
                            r: 0.7,
                            g: 0.7,
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
            self.textures
                .iter()
                .enumerate()
                .for_each(|(index, &texture)| {
                    render_pass.set_bind_group(index as u32, &texture.bind_group, &[]);
                });
            // render_pass.set_bind_group(0, self.sprite.bind_group(), &[]);
            render_pass.set_bind_group(self.textures.len() as u32, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
            render_pass.draw_indexed(0..all_indices.len() as u32, 0, 0..1);
        }

        context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
