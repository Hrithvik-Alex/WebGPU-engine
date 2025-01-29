use std::cell::RefCell;
use std::convert;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use crate::camera;
use crate::component;
use crate::context;
use crate::game;
use crate::gui;
use crate::model;
use crate::model::ModelVertex2d;
use crate::model::Vertex;
use crate::sprite;
use crate::texture;
use crate::uniform;
use crate::uniform::LightUniform;
use crate::utils;
use crate::wgsl_preprocessor;
use cgmath::num_traits::ToPrimitive;
use cgmath::ElementWise;

use cgmath::Vector2;
use egui_winit::winit::window::Window;
use log::debug;
use wgpu::core::identity;
use wgpu::util::DeviceExt;
use wgpu::BindGroupDescriptor;
use wgpu::BufferSlice;
use wgpu::StencilState;

pub struct RenderSystem {
    // positions: Vec<&'a component::PositionComponent>,
    // vertex_arrays: Vec<&'a component::VertexArrayComponent>,
    // textures: Vec<&'a texture::Texture>,
    // context: &'a context::Context<'a>,
    orig_render_pipeline: wgpu::RenderPipeline,
    collectible_render_pipeline: wgpu::RenderPipeline,
    debug_render_pipeline: wgpu::RenderPipeline,
    // debug_bind_group_layout: wgpu::PipelineLayout,
    wireframe_render_pipeline: wgpu::RenderPipeline,
    wireframe_bind_group_layout: wgpu::BindGroupLayout,
    post_standard_render_pipeline: wgpu::RenderPipeline,
    post_popup_render_pipeline: wgpu::RenderPipeline,
    post_standard_bind_group_layout: wgpu::BindGroupLayout,
    post_popup_bind_group_layout: wgpu::BindGroupLayout,
    light_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: wgpu::BindGroup,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    stencil_compute_pipeline: wgpu::ComputePipeline,
    stencil_compute_bind_group_layout: wgpu::BindGroupLayout,
    depth_stencil: texture::TextureBasic,
}

struct ModelBuffer {
    vertices: Vec<ModelVertex2d>,
    indices: Vec<u32>,
}

struct PipelineInfo<'a> {
pos: &'a component::PositionComponent,
v_arr: &'a component::VertexArrayComponent,
light: &'a Option<uniform::LightComponent>,
metadata: &'a component::MetadataComponent,
}

impl RenderSystem {
    const STANDARD_STENCIL_FACE_STATE: wgpu::StencilFaceState = wgpu::StencilFaceState {
        compare: wgpu::CompareFunction::Always,
        fail_op: wgpu::StencilOperation::Keep,
        depth_fail_op: wgpu::StencilOperation::Keep,
        pass_op: wgpu::StencilOperation::IncrementClamp,
    };
    const STANDARD_STENCIL_STATE: wgpu::StencilState = wgpu::StencilState {
        front: Self::STANDARD_STENCIL_FACE_STATE,
        back: Self::STANDARD_STENCIL_FACE_STATE,
        // Applied to values being read from the buffer
        read_mask: 0xff,
        // Applied to values before being written to the buffer
        write_mask: 0xff,
    };

    const OUTLINE_SCALE_FACTOR: f32 = 1.1;
    const WORKGROUP_SIZE_X: u32 = 8;
    const WORKGROUP_SIZE_Y: u32 = 8;
    const AMBIENT_LIGHT_INTENSITY : f32 = 0.5;



    pub fn new(textures: &Vec<Arc<texture::Texture>>, context: &context::Context, wgsl_preprocessor: &wgsl_preprocessor::WgslPreprocessor) -> Self {
        // debug!("{:?}", camera_buffer);
        // debug!("{:?}", world_buffer);

        let uniform_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("uniform bind group layout"),
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
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let light_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("light bind group layout"),
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
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let (texture_bind_group_layout, texture_bind_group) =
            Self::create_texture_bindings(textures, context);
        let bind_group_layouts: Vec<&wgpu::BindGroupLayout> = vec![
            &uniform_bind_group_layout,
            &texture_bind_group_layout,
            &light_bind_group_layout,
        ];

        // bind_group_layouts.extend(textures.iter().map(|texture| &texture.bind_group_layout));

        let standard_shader: wgpu::ShaderModule =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("standard shader"),
                    source: wgpu::ShaderSource::Wgsl(wgsl_preprocessor.get_code("standard.wgsl".to_string()).into()),
                });
        
        let collectible_shader: wgpu::ShaderModule =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("hover shader"),
                    source: wgpu::ShaderSource::Wgsl(wgsl_preprocessor.get_code("hover.wgsl".to_string()).into()),
                });

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &bind_group_layouts,
                    push_constant_ranges: &[],
                });

        let orig_render_pipeline = Self::create_pipeline(
            "Original Render Pipeline",
            &context,
            &render_pipeline_layout,
            &standard_shader,
            Some(Self::STANDARD_STENCIL_STATE),
            &[model::ModelVertex2d::desc()],
            None,
        );

        let collectible_render_pipeline = Self::create_pipeline(
            "Collectible Render Pipeline",
            &context,
            &render_pipeline_layout,
            &collectible_shader,
            Some(Self::STANDARD_STENCIL_STATE),
            &[model::ModelVertex2d::desc()],
            None,
        );
        
        let depth_stencil = texture::TextureBasic::create_depth_texture(
            &context.device,
            &context.config,
            "depth_texture",
        );

        let (wireframe_render_pipeline, wireframe_bind_group_layout) =
            Self::create_wireframe_pipeline(context, &uniform_bind_group_layout, wgsl_preprocessor);


        let post_standard_shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("post standard shader"),
                source: wgpu::ShaderSource::Wgsl(wgsl_preprocessor.get_code("post_standard.wgsl".to_string()).into()),
            });
        let (post_standard_render_pipeline, post_standard_bind_group_layout) = Self::create_post_pipeline(context, &post_standard_shader);

        let post_popup_shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("post popup shader"),
                source: wgpu::ShaderSource::Wgsl(wgsl_preprocessor.get_code("post_popup.wgsl".to_string()).into()),
            });
        let (post_popup_render_pipeline, post_popup_bind_group_layout) = Self::create_post_pipeline(context, &post_popup_shader);

        let (debug_render_pipeline) = Self::create_debug_pipeline(
            context,
            &uniform_bind_group_layout,
            &texture_bind_group_layout,
            wgsl_preprocessor
        );

        let (stencil_compute_pipeline, stencil_compute_bind_group_layout) =
            Self::create_stencil_compute_pipeline(context, wgsl_preprocessor);

        Self {
            orig_render_pipeline,
            collectible_render_pipeline,
            debug_render_pipeline,
            // debug_bind_group_layout,
            wireframe_render_pipeline,
            wireframe_bind_group_layout,
            post_standard_render_pipeline,
            post_popup_render_pipeline,
            post_standard_bind_group_layout,
            post_popup_bind_group_layout,
            uniform_bind_group_layout,
            light_bind_group_layout,
            texture_bind_group,
            stencil_compute_pipeline,
            stencil_compute_bind_group_layout,
            depth_stencil,
        }
    }

    pub fn resize(&mut self, textures: &Vec<Arc<texture::Texture>>, context: &context::Context) {
        let (_, texture_bind_group) = Self::create_texture_bindings(textures, context);

        self.texture_bind_group = texture_bind_group;

        self.depth_stencil = texture::TextureBasic::create_depth_texture(
            &context.device,
            &context.config,
            "depth_texture",
        );
    }

    pub fn create_texture_bindings(
        textures: &Vec<Arc<texture::Texture>>,
        context: &context::Context,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let mut texture_bind_group_layout_entries = vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            // This should match the filterable field of the
            // corresponding Texture entry above.
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        }];

        let sampler = texture::TextureBasic::default_pixel_sampler(&context.device);

        let mut texture_bind_group_entries = vec![wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Sampler(&sampler),
        }];

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
                    binding: cur_len + 1,
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

        let texture_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("texture bind group layout"),
                    entries: &texture_bind_group_layout_entries,
                });

        let texture_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("texture bind group layout"),
                layout: &texture_bind_group_layout,
                entries: &texture_bind_group_entries,
            });

        (texture_bind_group_layout, texture_bind_group)
    }

    fn get_vertices_for_entity<'a>(
        pos: &'a component::PositionComponent,
        vertex_array: &'a component::VertexArrayComponent,
        sprite_sheets: &Vec<Rc<RefCell<sprite::SpriteSheet>>>,
        cur_len: usize,
    ) -> (
        impl Iterator<Item = ModelVertex2d> + 'a,
        impl Iterator<Item = u32> + 'a,
    ) {
        let max_x = vertex_array.tex_coords.iter().map(|tex| tex.x).fold(f32::MIN, |a, b| a.max(b));
        let min_x = vertex_array.tex_coords.iter().map(|tex| tex.x).fold(f32::MAX, |a, b| a.min(b));
        (
            vertex_array
                .vertices
                .iter()
                .zip(vertex_array.tex_coords.iter())
                .map(move |(vertex_pos, &tex_coord)| {
                    // TODO: hacky POS
                    let final_tex_coord = if vertex_array.is_flipped {
                        let tex_coord_x = if tex_coord.x == max_x {min_x} else {max_x};
                        Vector2::new(tex_coord_x, tex_coord.y)
                    } else {
                        tex_coord
                    };

                    let twod_coords = (vertex_pos.mul_element_wise(pos.scale)) + pos.position;

                    model::ModelVertex2d {
                        position: cgmath::Vector3::new(
                            twod_coords.x,
                            twod_coords.y,
                            vertex_array.z_value,
                        )
                        .into(),
                        tex_coords: final_tex_coord.into(),
                        normal_coords: final_tex_coord.into(), // TODO: maybe have to flip something here?
                        extra_info: (vertex_array.texture_index
                            + vertex_array.is_flipped as u32 * 256),
                    }
                }),
            vertex_array
                .indices
                .iter()
                .map(move |index| cur_len.clone() as u32 + index),
        )
    }

    fn get_vertex_and_lighting_data(
pipeline_infos: Vec<PipelineInfo>,
sprite_sheets: &Vec<Rc<RefCell<sprite::SpriteSheet>>>,
    ) -> (ModelBuffer, Vec<LightUniform>, usize) {
        let (vertices, indices, light_uniforms, len) = 
        
        pipeline_infos.iter()
            .fold(
                (Vec::new(), Vec::new(), Vec::new(), 0),
                |(mut vertices, mut indices, mut light_uniforms, i),
                PipelineInfo {pos,  v_arr, light,metadata}| {
                    let cur_len = vertices.len();
                    let (entity_vertices, entity_indices) =
                        Self::get_vertices_for_entity(pos, v_arr, &sprite_sheets, cur_len);
                    vertices.extend(entity_vertices);
                    indices.extend(entity_indices);

                    if let Some(light) = light {
                        light_uniforms.push(uniform::LightUniform {
                            position: cgmath::Vector3::new(
                                pos.position.x,
                                pos.position.y,
                                v_arr.z_value,
                            )
                            .into(),
                            color: light.color.into(),
                            linear_dropoff: light.linear_dropoff,
                            quadratic_dropoff: light.quadratic_dropoff,
                            ambient_strength: light.ambient_strength,
                            diffuse_strength: light.diffuse_strength,
                            padding: [0.0, 0.0],
                        });
                    }

                    (vertices, indices, light_uniforms, i + 1)
                },
            );
            (ModelBuffer{ vertices, indices}, light_uniforms, len)
    }

    pub fn render(
        &self,
        positions: &component::EntityMap<component::PositionComponent>,
        vertex_arrays: &component::EntityMap<component::VertexArrayComponent>,
        lights: &component::EntityMap<uniform::LightComponent>,
        metadata_components: &component::EntityMap<component::MetadataComponent>,
        sprite_sheets: &Vec<Rc<RefCell<sprite::SpriteSheet>>>,
        context: &context::Context,
        gui: &mut gui::Gui,
        window: Arc<Window>,
        add_debug_pass: bool,
        time_elapsed: Duration,
        world_uniform: &uniform::WorldUniform,
        camera: &camera::OrthographicCamera,
        gui_info: &gui::GuiInfo,
        game_mode: &mut game::GameMode
    ) -> Result<(), wgpu::SurfaceError> {

            // debug!("BOO {:?}", window.inner_size());
        // let mut all_vertices: Vec<ModelVertex2d> = vec![];
        // let mut all_indices: Vec<u32> = vec![];
        let camera_buffer = camera.get_buffer(&context.device);
        let world_buffer = world_uniform.get_buffer(&context.device);
        let screen_resolution_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Screen Resloution Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[
                        context.config.width as f32,
                        context.config.height as f32,
                    ]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });



       let time_buffer = context
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Time Uniform Buffer"),
                        contents: bytemuck::cast_slice(&[time_elapsed.as_secs_f32()]),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });


        let uniform_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.uniform_bind_group_layout,
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
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: screen_resolution_buffer.as_entire_binding(),
                    },
                    
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: time_buffer.as_entire_binding(),
                    },
                ],
                label: Some("uniform bind group"),
            });


        let (standard_pipeline_infos, collectible_pipeline_infos ): (Vec<_>, Vec<_>) = 
        utils::zip4_entities(positions, vertex_arrays, lights, metadata_components)
        .filter_map(|(_, pos, v_arr, light, metadata)| {
            match (pos, v_arr, metadata ){
                (Some(pos), Some(v_arr), Some(metadata)) => {
                    Some(PipelineInfo {pos: pos, v_arr: v_arr, light: light, metadata: metadata})
                }
                _ => None
            }
        })
        .partition(|pipeline_info| {
           pipeline_info.v_arr.shader_type == component::ShaderType::STANDARD 
        });

        let (standard_model_buffer, standard_light_uniforms, _) = Self::get_vertex_and_lighting_data(
standard_pipeline_infos, sprite_sheets
        );

        let (collectible_model_buffer, collectible_light_uniforms, _) = Self::get_vertex_and_lighting_data(
            collectible_pipeline_infos, sprite_sheets
                    );


        let num_vertices = standard_model_buffer.vertices.len();
        let standard_num_indices = standard_model_buffer.indices.len();

        // debug!("{:?}", light_uniforms);
        // debug!("{:?}", all_vertices);
        // debug!("{:?}", all_indices.len());


        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let frame_buffer_a =
            texture::TextureBasic::create_basic(&context.device, &context.config, "frame buffer a");

        // let frame_buffer_b =
        //     texture::TextureBasic::create_basic(&context.device, &context.config, "frame buffer b");



        let light_uniforms_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Light Uniforms Buffer"),
                    contents: bytemuck::cast_slice(&[standard_light_uniforms.as_slice(), collectible_light_uniforms.as_slice()].concat()),
                    usage: wgpu::BufferUsages::STORAGE,
                });

        let light_len_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Light Len Buffer"),
                    contents: bytemuck::cast_slice(&[standard_light_uniforms.len() + collectible_light_uniforms.len()]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
        
        let ambient_light_intensity_buffer = 
        context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Len Buffer"),
            contents: bytemuck::cast_slice(&[Self::AMBIENT_LIGHT_INTENSITY]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        // debug!("{:?}", bytemuck::cast_slice::<uniform::LightUniform, f32>(&light_uniforms));

        let light_bind_group = context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("light bind group"),
            layout: &self.light_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_len_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: ambient_light_intensity_buffer.as_entire_binding(),
                },       
                     ],
        });

        let output = context.surface.get_current_texture()?;

        let surface_tex = &output.texture;
        let surface_view = surface_tex.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Original Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: 
                    // &surface_view,
                    &frame_buffer_a.view,
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

            let standard_vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&standard_model_buffer.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            });

        let standard_index_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&standard_model_buffer.indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::STORAGE,
            });
        


            render_pass.set_pipeline(&self.orig_render_pipeline);

            render_pass.set_bind_group(0, &uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bind_group, &[]);

            render_pass.set_bind_group(2, &light_bind_group, &[]);

            render_pass.set_vertex_buffer(0, standard_vertex_buffer.slice(..));
            render_pass.set_index_buffer(standard_index_buffer.slice(..), wgpu::IndexFormat::Uint32); // 1.
            render_pass.draw_indexed(0..standard_model_buffer.indices.len() as u32, 0, 0..1);

            let collectible_vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&collectible_model_buffer.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            });

        let collectible_index_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&collectible_model_buffer.indices),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::STORAGE,
            });

            render_pass.set_pipeline(&self.collectible_render_pipeline);

            render_pass.set_bind_group(0, &uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bind_group, &[]);

            render_pass.set_bind_group(2, &light_bind_group, &[]);

            render_pass.set_vertex_buffer(0, collectible_vertex_buffer.slice(..));
            render_pass.set_index_buffer(collectible_index_buffer.slice(..), wgpu::IndexFormat::Uint32); // 1.
            render_pass.draw_indexed(0..collectible_model_buffer.indices.len() as u32, 0, 0..1);




            // render_pass.set_pipeline(&self.wireframe_render_pipeline);

            // let wireframe_bind_group =
            //     context
            //         .device
            //         .create_bind_group(&wgpu::BindGroupDescriptor {
            //             label: Some("wireframe bind group"),
            //             layout: &self.wireframe_bind_group_layout,
            //             entries: &[
            //                 wgpu::BindGroupEntry {
            //                     binding: 0,
            //                     resource: standard_vertex_buffer.as_entire_binding(),
            //                 },
            //                 wgpu::BindGroupEntry {
            //                     binding: 1,
            //                     resource: standard_index_buffer.as_entire_binding(),
            //                 },
            //             ],
            //         });
            // render_pass.set_bind_group(1, &wireframe_bind_group, &[]);
            // render_pass.draw(0..standard_num_indices as u32 * 2, 0..1); // TODO: slightly overdraws 6 instead of 5 edges per, maybe optimize?
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Post Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    // &output
                    //     .texture
                    //     .create_view(&wgpu::TextureViewDescriptor::default()),
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
                depth_stencil_attachment: None,
                //  Some(wgpu::RenderPassDepthStencilAttachment {
                //     view: &self.depth_stencil.view,
                //     depth_ops: Some(wgpu::Operations {
                //         load: wgpu::LoadOp::Load,
                //         store: wgpu::StoreOp::Store,
                //     }),
                //     stencil_ops: Some(wgpu::Operations {
                //         load: wgpu::LoadOp::Load,
                //         store: wgpu::StoreOp::Store,
                //     }),
                // }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            let post_bind_group_layout = match game_mode {
                game::GameMode::STANDARD => &self.post_standard_bind_group_layout,
                game::GameMode::POPUP => &self.post_popup_bind_group_layout,
            };

            let post_render_pipeline = match game_mode {
                game::GameMode::STANDARD => &self.post_standard_render_pipeline,
                game::GameMode::POPUP => &self.post_popup_render_pipeline,
            };

            let bind_group = context
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("post bind group"),
                    layout: post_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&frame_buffer_a.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&frame_buffer_a.sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: time_buffer.as_entire_binding(),
                        },
                    ],
                });
            render_pass.set_pipeline(post_render_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }

        // let width = surface_tex.width();
        // let height = surface_tex.height();

        // let width_div_4_256_aligned = (((width) + 255) / 256) * 256 / 4;
        // let emtpy_array_r = (0..(width_div_4_256_aligned * height))
        //     .map(|_| 0 as u32)
        //     .collect::<Vec<u32>>();
        // let read_buffer = context
        //     .device
        //     .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //         label: Some("Intermediate Stencil Compute Buffer map"),
        //         contents: bytemuck::cast_slice(&emtpy_array_r),
        //         usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        //     });
        if add_debug_pass {
            let (outline_vertices, outline_indices) =
                utils::zip3_entities(positions, vertex_arrays, metadata_components)
                    .filter_map(|(_, pos, vertex_array, metadata)| {
                        pos.as_ref().and_then(|pos| {
                            vertex_array.as_ref().and_then(|vertex_array| {
                                metadata.as_ref().unwrap().should_outline().then(|| {
                                    let mut new_pos = pos.clone();
                                    new_pos.scale_outward(cgmath::Vector2::new(
                                        Self::OUTLINE_SCALE_FACTOR,
                                        Self::OUTLINE_SCALE_FACTOR,
                                    ));
                                    (new_pos, vertex_array)
                                })
                            })
                        })
                    })
                    .fold(
                        (Vec::new(), Vec::new()),
                        |(mut vertices, mut indices), (pos, vertex_array)| {
                            let cur_len = vertices.len();
                            let (v, i) = Self::get_vertices_for_entity(&pos, vertex_array, sprite_sheets, cur_len);
                            vertices.extend(v);
                            indices.extend(i);
                            (vertices, indices)
                        },
                    );

            // debug!("{:?}", all_vertices);
            // debug!("{:?}", outline_vertices);

            let debug_vertex_buffer =
                context
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&outline_vertices),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
                    });

            let debug_index_buffer =
                context
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&outline_indices),
                        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::STORAGE,
                    });

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Debug Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
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
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.debug_render_pipeline);
            render_pass.set_bind_group(0, &uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
            render_pass.set_stencil_reference(2);
            render_pass.set_vertex_buffer(0, debug_vertex_buffer.slice(..));
            render_pass.set_index_buffer(debug_index_buffer.slice(..), wgpu::IndexFormat::Uint32); // 1.
            render_pass.draw_indexed(0..outline_indices.len() as u32, 0, 0..1);
        }
        // let buffer = self.debug_stencil(context, &mut encoder, surface_tex);

        // encoder.copy_buffer_to_buffer(
        //     &buffer,
        //     0,
        //     &read_buffer,
        //     0,
        //     (width_div_4_256_aligned * height) as u64,
        // );

        // encoder.copy_texture_to_buffer(
        //     wgpu::ImageCopyTexture {
        //         texture: &self.depth_stencil.texture,
        //         mip_level: 0,
        //         origin: wgpu::Origin3d::ZERO,
        //         aspect: wgpu::TextureAspect::StencilOnly,
        //     },
        //     wgpu::ImageCopyBuffer {
        //         buffer: &read_buffer,
        //         layout: wgpu::ImageDataLayout {
        //             offset: 0,
        //             bytes_per_row: Some(
        //                 width_div_4_256_aligned * 4 * std::mem::size_of::<u8>() as u32,
        //             ),
        //             rows_per_image: Some(height),
        //         },
        //     },
        //     wgpu::Extent3d {
        //         width,
        //         height,
        //         depth_or_array_layers: 1,
        //     },
        // );

        gui.draw(&context, &mut encoder, window, &surface_view, gui_info, game_mode);

        context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // let buffer_slice = read_buffer.slice(..);
        // let (sender, receiver) = flume::bounded(1);

        // buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        // context
        //     .device
        //     .poll(wgpu::Maintain::wait())
        //     .panic_on_timeout();

        // if let Ok(Ok(())) = pollster::block_on(receiver.recv_async()) {
        //     // Gets contents of buffer
        //     let data = buffer_slice.get_mapped_range();
        //     // Since contents are got in bytes, this converts these bytes back to u32
        //     let result: Vec<u8> = bytemuck::cast_slice(&data).to_vec();
        //     debug!("{:?}", width_div_4_256_aligned);
        //     debug!("{:?}", result.iter().skip(4800).position(|&x| x == 0));
        //     debug!("{:?}", &result[6400..8000]);
        //     debug!("{:?}", result.iter().filter(|&&v| v == 0).count());
        //     debug!("{:?}", result.iter().filter(|&&v| v == 1).count());
        //     debug!("{:?}", result.iter().filter(|&&v| v == 2).count());
        //     debug!("{:?}", result.iter().filter(|&&v| v == 3).count());
        //     debug!("{:?}", result.iter().filter(|&&v| v == 4).count());
        //     // With the current interface, we have to make sure all mapped views are
        //     // dropped before we unmap the buffer.
        //     drop(data);
        // } else {
        //     panic!("failed to run compute on gpu!")
        // }

        Ok(())
    }

    // WARNING: only works when width is a multiple of 256, and since its debug not worth to make this work generally right now.
    fn debug_stencil(
        &self,
        context: &context::Context,
        encoder: &mut wgpu::CommandEncoder,
        surface_tex: &wgpu::Texture,
    ) -> wgpu::Buffer {
        let width = surface_tex.width();
        let height = surface_tex.height();

        let width_div_4_256_aligned = (((width) + 255) / 256) * 256 / 4;
        let emtpy_array_r = (0..(width_div_4_256_aligned * height))
            .map(|_| 0 as u32)
            .collect::<Vec<u32>>();
        let buffer_r = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Intermediate Stencil Compute Buffer read"),
                contents: bytemuck::cast_slice(&emtpy_array_r),
                usage: wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
            });
        // debug!("size: {:?}", self.depth_stencil.texture.format());

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.depth_stencil.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::StencilOnly,
            },
            wgpu::ImageCopyBuffer {
                buffer: &buffer_r,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        width_div_4_256_aligned * 4 * std::mem::size_of::<u8>() as u32,
                    ),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let width_256_aligned = (((width * 4) + 255) / 256) * 256 / 4;
        let emtpy_array_w = (0..(width_256_aligned * height))
            .map(|_| 0 as u32)
            .collect::<Vec<u32>>();
        let buffer_w = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Intermediate Stencil Compute Buffer write"),
                contents: bytemuck::cast_slice(&emtpy_array_w),
                usage: wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
            });

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Stencil compute bind group"),
                layout: &self.stencil_compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            buffer_r.as_entire_buffer_binding(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(
                            buffer_w.as_entire_buffer_binding(),
                        ),
                    },
                ],
            });
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("depth stencil compute pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(&self.stencil_compute_pipeline);
            compute_pass.set_bind_group(0, &bind_group, &[]);
            compute_pass.dispatch_workgroups(
                width / Self::WORKGROUP_SIZE_X + 1,
                height / Self::WORKGROUP_SIZE_Y + 1,
                1,
            );
        }

        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &buffer_w,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(width_256_aligned * std::mem::size_of::<u32>() as u32),
                    rows_per_image: Some(height),
                },
            },
            wgpu::ImageCopyTexture {
                texture: &surface_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        buffer_r
    }

    fn create_stencil_compute_pipeline(
        context: &context::Context,
        wgsl_preprocessor: &wgsl_preprocessor::WgslPreprocessor
    ) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Depth to Color Compute Shader"),
                source: wgpu::ShaderSource::Wgsl(wgsl_preprocessor.get_code("compute.wgsl".to_string()).into()),
            });

        // Create a bind group layout
        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Stencil Compute Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        (
            context
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Stencil Compute Pipeline"),
                    layout: Some(&pipeline_layout),
                    module: &shader,
                    entry_point: "convert_depth_to_color",
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                }),
            bind_group_layout,
        )
    }

    fn create_pipeline(
        name: &str,
        context: &context::Context,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        stencil_state: Option<StencilState>,
        buffers: &[wgpu::VertexBufferLayout<'_>],
        topology: Option<wgpu::PrimitiveTopology>,
    ) -> wgpu::RenderPipeline {
        let topology = topology.unwrap_or(wgpu::PrimitiveTopology::TriangleList);

        context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(name),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers,
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.config.format,
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology,
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
                depth_stencil: stencil_state.map(|stencil_state| wgpu::DepthStencilState {
                    format: texture::TextureBasic::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    bias: wgpu::DepthBiasState::default(),
                    stencil: stencil_state,
                }),
                // None,
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

    fn create_wireframe_pipeline(
        context: &context::Context,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        wgsl_preprocessor: &wgsl_preprocessor::WgslPreprocessor
    ) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
        let wireframe_shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("wireframe shader"),
                source: wgpu::ShaderSource::Wgsl(wgsl_preprocessor.get_code("wireframe.wgsl".to_string()).into()),
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

        (
            Self::create_pipeline(
                "Wireframe Pipeline",
                &context,
                &render_pipeline_layout,
                &wireframe_shader,
                Some(Self::STANDARD_STENCIL_STATE),
                &[],
                Some(wgpu::PrimitiveTopology::LineList),
            ),
            bind_group_layout,
        )
    }

    fn create_post_pipeline(
        context: &context::Context,
        post_shader: &wgpu::ShaderModule
    ) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {


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
                    label: Some("post Render Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });

        (
            Self::create_pipeline(
                "Post Pipeline",
                &context,
                &render_pipeline_layout,
                &post_shader,
                None,
                &[],
                None,
            ),
            bind_group_layout,
        )
    }

    fn create_debug_pipeline(
        context: &context::Context,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        wgsl_preprocessor: &wgsl_preprocessor::WgslPreprocessor
    ) -> wgpu::RenderPipeline {
        let debug_shader: wgpu::ShaderModule =
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("outline"),
                    source: wgpu::ShaderSource::Wgsl(wgsl_preprocessor.get_code("outline.wgsl".to_string()).into()),
                });

        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Debug Render Pipeline Layout"),
                    bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let stencil_face_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Greater,
            fail_op: wgpu::StencilOperation::Zero,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Keep,
        };

        Self::create_pipeline(
            "Debug Render Pipeline",
            &context,
            &render_pipeline_layout,
            &debug_shader,
            Some(wgpu::StencilState {
                front: stencil_face_state,
                back: stencil_face_state,
                // Applied to values being read from the buffer
                read_mask: 0xff,
                // Applied to values before being written to the buffer
                write_mask: 0xff,
            }),
            &[model::ModelVertex2d::desc()],
            None,
        )
    }
}
