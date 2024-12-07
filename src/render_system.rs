use std::sync::Arc;

use crate::camera;
use crate::component;
use crate::context;
use crate::model;
use crate::model::Vertex;
use crate::texture;
use cgmath::num_traits::ToPrimitive;
use cgmath::ElementWise;

use cgmath::Vector2;
use log::debug;
use wgpu::util::DeviceExt;

pub struct RenderSystem {
    // positions: Vec<&'a component::PositionComponent>,
    // vertex_arrays: Vec<&'a component::VertexArrayComponent>,
    // textures: Vec<&'a texture::Texture>,
    // context: &'a context::Context<'a>,
    orig_render_pipeline: wgpu::RenderPipeline,
    debug_render_pipeline: wgpu::RenderPipeline,
    uniform_bind_group: wgpu::BindGroup,
    depth_stencil: texture::TextureBasic,
}

impl RenderSystem {
    pub fn new(
        textures: &Vec<Arc<texture::Texture>>,
        context: &context::Context,
        world_uniform: &component::WorldUniform,
        camera: &camera::OrthographicCamera,
    ) -> Self {
        let camera_buffer = camera.get_buffer(&context.device);
        let world_buffer = world_uniform.get_buffer(&context.device);

        // debug!("{:?}", camera_buffer);
        // debug!("{:?}", world_buffer);

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


        let mut bind_group_layouts: Vec<&wgpu::BindGroupLayout> = vec![&uniform_bind_group_layout];

        bind_group_layouts.extend(textures.iter().map(|texture| &texture.bind_group_layout));

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &bind_group_layouts,
                    push_constant_ranges: &[],
                });

        let stencil_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Always,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::IncrementClamp,
        };

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
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: texture::TextureBasic::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        bias: wgpu::DepthBiasState::default(),
                        stencil: wgpu::StencilState {
                            front: stencil_state,
                            back: stencil_state,
                            // Applied to values being read from the buffer
                            read_mask: 0xff,
                            // Applied to values before being written to the buffer
                            write_mask: 0xff,
                        },
                    }),
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

        let orig_render_pipeline =
            create_render_pipeline("Original Render Pipeline", "vs_main", "fs_main");

        let debug_render_pipeline =
            create_render_pipeline("Debug Render Pipeline", "vs_main", "fs_main");
      

        let depth_stencil = texture::TextureBasic::create_depth_texture(
            &context.device,
            &context.config,
            "depth_texture",
        );

        Self {
            orig_render_pipeline,
            debug_render_pipeline,
            uniform_bind_group,
            depth_stencil,
        }
    }

    fn create_wireframe_pipeline(context: &context::Context,) -> wgpu::RenderPipeline {

        let wireframe_shader = 
        context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("wireframe shader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("wireframe.wgsl").into()),
                }); 


        context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Wireframe Pipeline"),
            layout: ,// TODO
            vertex: wgpu::VertexState {
                module: &wireframe_shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: ,// TODO
            },
            fragment: Some(wgpu::FragmentState {
                module: &wireframe_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.config.format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None, 
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
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
    }

    pub fn render(
        &self,
        positions: &component::EntityMap<component::PositionComponent>,
        vertex_arrays: &component::EntityMap<component::VertexArrayComponent>,
        textures: &Vec<Arc<texture::Texture>>,
        context: &context::Context,
        add_debug_pass: bool,
    ) -> Result<(), wgpu::SurfaceError> {
        // let mut all_vertices: Vec<ModelVertex2d> = vec![];
        // let mut all_indices: Vec<u32> = vec![];

        let (all_vertices, all_indices, _) = positions
            .iter()
            .zip(vertex_arrays.iter())
            .filter_map(|((_, opt1), (_, opt2))| {
                opt1.as_ref()
                    .and_then(|v1| opt2.as_ref().map(|v2| (v1, v2)))
            })
            .fold(
                (Vec::new(), Vec::new(), 0),
                |(mut vertices, mut indices, i), (pos, vertex_array)| {
                    vertices.extend(
                        vertex_array
                            .vertices
                            .iter()
                            .zip(vertex_array.tex_coords.iter())
                            .map(|(vertex_pos, &tex_coord)| {
                                let final_tex_coord = if vertex_array.is_flipped {
                                    Vector2::new(1. - tex_coord.x, tex_coord.y)
                                } else {
                                    tex_coord
                                };

                                model::ModelVertex2d {
                                    position: ((vertex_pos.mul_element_wise(pos.scale))
                                        + pos.position)
                                        .into(),
                                    tex_coords: final_tex_coord.into(),
                                    normal_coords: final_tex_coord.into(),
                                    texture: vertex_array.texture_index,
                                }
                            }),
                    );
                    indices.extend(vertex_array.indices.iter().map(|index| 4 * i + index));
                    (vertices, indices, i + 1)
                },
            );
        // debug!("{:?}", all_vertices);
        // debug!("{:?}", all_indices);
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
                label: Some("Original Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_stencil.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&self.orig_render_pipeline);

            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            textures.iter().enumerate().for_each(|(index, texture)| {
                render_pass.set_bind_group(index as u32 + 1, &texture.bind_group, &[]);
            });
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32); // 1.
            render_pass.draw_indexed(0..all_indices.len() as u32, 0, 0..1);
        }

        if add_debug_pass {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Debug Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_stencil.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.debug_render_pipeline);
        }

        context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
