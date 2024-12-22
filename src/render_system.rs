use std::sync::Arc;
use std::time::Duration;

use crate::camera;
use crate::component;
use crate::context;
use crate::model;
use crate::model::Vertex;
use crate::texture;
use crate::uniform;
use cgmath::num_traits::ToPrimitive;
use cgmath::ElementWise;

use cgmath::Vector2;
use log::debug;
use wgpu::util::DeviceExt;
use wgpu::BindGroupDescriptor;

pub struct RenderSystem {
    // positions: Vec<&'a component::PositionComponent>,
    // vertex_arrays: Vec<&'a component::VertexArrayComponent>,
    // textures: Vec<&'a texture::Texture>,
    // context: &'a context::Context<'a>,
    orig_render_pipeline: wgpu::RenderPipeline,
    debug_render_pipeline: wgpu::RenderPipeline,
    wireframe_render_pipeline: wgpu::RenderPipeline,
    wireframe_bind_group_layout: wgpu::BindGroupLayout,
    post_render_pipeline: wgpu::RenderPipeline,
    post_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group: wgpu::BindGroup,
    storage_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,
    depth_stencil: texture::TextureBasic,
}

impl RenderSystem {
    pub fn new(
        textures: &Vec<Arc<texture::Texture>>,
        context: &context::Context,
        world_uniform: &uniform::WorldUniform,
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
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX  | wgpu::ShaderStages::FRAGMENT,
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


        let storage_bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("storage bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,     
               },
               wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,     
           },
            ]
        });



        let mut texture_bind_group_layout_entries = vec![
            wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
        ];

        let sampler = texture::TextureBasic::default_pixel_sampler(&context.device);

        let mut texture_bind_group_entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                    },
        ];

        for i in 0..textures.len() {
            let texture = &textures[i];
            let cur_len = texture_bind_group_layout_entries.len() as u32;


            texture_bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: cur_len,

                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            });

            texture_bind_group_entries.push(wgpu::BindGroupEntry {
                        binding: cur_len,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    });

            if let Some((_, normal_view)) = &texture.normal_info {
                texture_bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
                    binding: cur_len  + 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                }); 

                texture_bind_group_entries.push(wgpu::BindGroupEntry {
                    binding: cur_len + 1,
                    resource: wgpu::BindingResource::TextureView(&normal_view),
                });
            }
        }

        let texture_bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture bind group layout"),
            entries: &texture_bind_group_layout_entries
    });

    let texture_bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("texture bind group layout"),
        layout: &texture_bind_group_layout,
        entries: &texture_bind_group_entries
});

        
        let bind_group_layouts: Vec<&wgpu::BindGroupLayout> = vec![&uniform_bind_group_layout, &storage_bind_group_layout, &texture_bind_group_layout];

        // bind_group_layouts.extend(textures.iter().map(|texture| &texture.bind_group_layout));

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
                    depth_stencil: 
                    // None,
                     Some(wgpu::DepthStencilState {
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

        let (wireframe_render_pipeline, wireframe_bind_group_layout) =
            Self::create_wireframe_pipeline(context, &uniform_bind_group_layout);

        let (post_render_pipeline, post_bind_group_layout) = Self::create_post_pipeline(context);

        Self {
            orig_render_pipeline,
            debug_render_pipeline,
            wireframe_render_pipeline,
            wireframe_bind_group_layout,
            post_render_pipeline,
            post_bind_group_layout,
            uniform_bind_group,
            storage_bind_group_layout,
            texture_bind_group,
            depth_stencil,
        }
    }

    fn create_wireframe_pipeline(
        context: &context::Context,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
        let wireframe_shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("wireframe shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("wireframe.wgsl").into()),
            });

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("wireframe bind group layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Wireframe Render Pipeline Layout"),
                    bind_group_layouts: &[&uniform_bind_group_layout, &bind_group_layout],
                    push_constant_ranges: &[],
                });

        let stencil_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Always,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::IncrementClamp,
        };

        (
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Wireframe Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &wireframe_shader,
                        entry_point: "vs_main",
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[], // TODO
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
                    depth_stencil: 
                    // None,
                     Some(wgpu::DepthStencilState {
                        format: texture::TextureBasic::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        bias: wgpu::DepthBiasState {
                            // TODO: understand what this does
                            constant: 1,
                            slope_scale: 0.5,
                            clamp: 1.,
                        },
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
                }),
            bind_group_layout,
        )
    }

    fn create_post_pipeline(
        context: &context::Context,
    ) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
        let post_shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("post shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("post.wgsl").into()),
            });

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("post bind group layout"),
                    entries: &[
                        // TODO: move into BasicTexture?
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer { 
                                ty: wgpu::BufferBindingType::Uniform, 
                                has_dynamic_offset: false, 
                                min_binding_size: None },
                            count: None
                        }
                    ],
                });

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("post Render Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        let stencil_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Always,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::IncrementClamp,
        };

        (
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("post Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &post_shader,
                        entry_point: "vs_main",
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &[], // TODO
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &post_shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: context.config.format,
                            blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: 
                    //None,
                    Some(wgpu::DepthStencilState {
                        format: texture::TextureBasic::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        bias: wgpu::DepthBiasState {
                            // TODO: understand what this does
                            constant: 1,
                            slope_scale: 0.5,
                            clamp: 1.,
                        },
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
                }),
            bind_group_layout,
        )
    }

    pub fn render(
        &self,
        positions: &component::EntityMap<component::PositionComponent>,
        vertex_arrays: &component::EntityMap<component::VertexArrayComponent>,
        lights: &component::EntityMap<uniform::LightComponent>,
        textures: &Vec<Arc<texture::Texture>>,
        context: &context::Context,
        add_debug_pass: bool,
        time_elapsed: Duration,
    ) -> Result<(), wgpu::SurfaceError> {
        // let mut all_vertices: Vec<ModelVertex2d> = vec![];
        // let mut all_indices: Vec<u32> = vec![];

        let (all_vertices, all_indices, light_uniforms, _) = positions
            .iter()
            .zip(vertex_arrays.iter())
            .filter_map(|((_, opt1), (_, opt2))| {
                opt1.as_ref()
                    .and_then(|v1| opt2.as_ref().map(|v2| (v1, v2)))
            }).zip(lights.iter())
            .fold(
                (Vec::new(), Vec::new(), Vec::new(), 0),
                |(mut vertices, mut indices, mut light_uniforms, i), (( pos, vertex_array ), ( _,light ))| {
                    let cur_len = vertices.len();
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

                                let twod_coords = ((vertex_pos.mul_element_wise(pos.scale))
                                + pos.position);

                                model::ModelVertex2d {
                                    position: cgmath::Vector3::new(twod_coords.x, twod_coords.y, vertex_array.z_value)
                                        .into(),
                                    tex_coords: final_tex_coord.into(),
                                    normal_coords: final_tex_coord.into(), // TODO: maybe have to flip something here?
                                    extra_info: (vertex_array.texture_index + vertex_array.is_flipped as u32 * 256),
                                }
                            }),
                    );
                    indices.extend(vertex_array.indices.iter().map(|index| cur_len as u32 + index));

                    if let Some(light) = light {
                        light_uniforms.push(uniform::LightUniform {
                            position: cgmath::Vector3::new(pos.position.x, pos.position.y, vertex_array.z_value).into(),
                            color: light.color.into(),
                            linear_dropoff: light.linear_dropoff,
                            quadratic_dropoff: light.quadratic_dropoff,
                            ambient_strength: light.ambient_strength,
                            diffuse_strength: light.diffuse_strength,
                            padding: [0.0,0.0]
                        });
                    }

                    (vertices, indices, light_uniforms, i + 1)
                },
            );

        let num_vertices = all_vertices.len();
        let num_indices = all_indices.len();

        // debug!("{:?}", light_uniforms);
        // debug!("{:?}", all_vertices);
        // debug!("{:?}", all_indices.len());
        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&all_vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            });

        let index_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&all_indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::STORAGE,
            });

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let frame_buffer =
            texture::TextureBasic::create_basic(&context.device, &context.config, "frame buffer");


        let time_uniform = uniform::TimeUniform { time: (time_elapsed.as_millis() % u32::MAX as u128) as f32 };
        let time_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time Uniform Buffer"),
            contents: bytemuck::cast_slice(&[time_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }); 

        let light_uniforms_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Uniforms Buffer"),
            contents: bytemuck::cast_slice(&light_uniforms),
            usage: wgpu::BufferUsages::STORAGE,
        }); 

        let light_len_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Len Buffer"),
            contents: bytemuck::cast_slice(&[light_uniforms.len()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }); 

        // debug!("{:?}", bytemuck::cast_slice::<uniform::LightUniform, f32>(&light_uniforms));

        let storage_bind_group = context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("storage bind group"),
            layout: &self.storage_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_uniforms_buffer.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_len_buffer.as_entire_binding()
                }
            ]
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Original Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_buffer.view,
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
                depth_stencil_attachment:
                // None,
                Some(wgpu::RenderPassDepthStencilAttachment {
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
            render_pass.set_bind_group(1, &storage_bind_group, &[]);
            render_pass.set_bind_group(2, &self.texture_bind_group, &[]);


            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32); // 1.
            render_pass.draw_indexed(0..all_indices.len() as u32, 0, 0..1);

            render_pass.set_pipeline(&self.wireframe_render_pipeline);

            let wireframe_bind_group =
                context
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("wireframe bind group"),
                        layout: &self.wireframe_bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: vertex_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: index_buffer.as_entire_binding(),
                            },
                        ],
                    });
            render_pass.set_bind_group(1, &wireframe_bind_group, &[]);
            render_pass.draw(0..num_indices as u32 * 2, 0..1); // TODO: slightly overdraws 6 instead of 5 edges per, maybe optimize?
        }

        let output = context.surface.get_current_texture()?;
        // let surface_view = output.texture;
        // .create_xview(&wgpu::TextureViewDescriptor::default());

        // encoder.copy_texture_to_texture(
        //     wgpu::ImageCopyTexture {
        //         texture: &frame_buffer.texture,
        //         mip_level: 0,
        //         origin: wgpu::Origin3d::ZERO,
        //         aspect: wgpu::TextureAspect::All,
        //     },
        //     wgpu::ImageCopyTexture {
        //         texture: &output.texture,
        //         mip_level: 0,
        //         origin: wgpu::Origin3d::ZERO,
        //         aspect: wgpu::TextureAspect::All,
        //     },
        //     frame_buffer.texture.size(),
        // );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Post Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load:
                        //  wgpu::LoadOp::Load,
                        wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.8,
                            g: 0.8,
                            b: 0.8,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment:
                //  None,
                 Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_stencil.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            let bind_group = context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("post bind group"),
                    layout: &self.post_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&frame_buffer.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&frame_buffer.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: time_buffer.as_entire_binding()
                        }
                    ],
                });
            render_pass.set_pipeline(&self.post_render_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        // if add_debug_pass {
        //     let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //         label: Some("Debug Render Pass"),
        //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //             view: &view,
        //             resolve_target: None,
        //             ops: wgpu::Operations {
        //                 load: wgpu::LoadOp::Clear(wgpu::Color {
        //                     r: 0.0,
        //                     g: 0.0,
        //                     b: 0.0,
        //                     a: 1.0,
        //                 }),
        //                 store: wgpu::StoreOp::Store,
        //             },
        //         })],
        //         depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
        //             view: &self.depth_stencil.view,
        //             depth_ops: Some(wgpu::Operations {
        //                 load: wgpu::LoadOp::Clear(1.0),
        //                 store: wgpu::StoreOp::Store,
        //             }),
        //             stencil_ops: Some(wgpu::Operations {
        //                 load: wgpu::LoadOp::Clear(0),
        //                 store: wgpu::StoreOp::Store,
        //             }),
        //         }),
        //         occlusion_query_set: None,
        //         timestamp_writes: None,
        //     });

        //     render_pass.set_pipeline(&self.debug_render_pipeline);
        // }

        context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
