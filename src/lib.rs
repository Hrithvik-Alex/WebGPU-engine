mod camera;
mod context;
mod model;
mod sprite;
mod texture;

use std::sync::Arc;

use model::Vertex;

use log::debug;
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalPosition,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use winit::window::Window;

struct State<'a> {
    context: context::Context<'a>,
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
    position: PhysicalPosition<f64>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    sprite_sheet: Arc<sprite::SpriteSheet>,
    sprite: sprite::Sprite,
    camera: camera::OrthographicCamera,
    // camera: camera::Camera,
    // projection: camera::Projection,
    camera_bind_group: wgpu::BindGroup,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    async fn new(window: &'a Window) -> State<'a> {
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
        ));
        let sprite = sprite::Sprite::new(&sprite_sheet);
        // let camera = camera::Camera::new(cgmath::Vector3::new(0.0, 0.0, 5.0));
        // let camera_buffer = camera.get_buffer(&context.device);

        // let projection =
        //     camera::Projection::new(size.width, size.height, cgmath::Deg(45.0), 0.1, 100.0, true);
        // debug!("proj:{:?}", (projection.calc_matrix()));
        // debug!("bruh:{:?}", size);

        // let projection_buffer = projection.get_buffer(&context.device);

        let camera = camera::OrthographicCamera::new(
            size.width,
            size.height,
            0.1,
            100.0,
            cgmath::Vector3::new(size.width as f32 / 2.0, size.height as f32 / 2.0, 1.0),
        );

        let camera_buffer = camera.get_buffer(&context.device);

        let camera_bind_group_layout =
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
                        // wgpu::BindGroupLayoutEntry {
                        //     binding: 1,
                        //     visibility: wgpu::ShaderStages::VERTEX,
                        //     ty: wgpu::BindingType::Buffer {
                        //         ty: wgpu::BufferBindingType::Uniform,
                        //         has_dynamic_offset: false,
                        //         min_binding_size: None,
                        //     },
                        //     count: None,
                        // },
                    ],
                });

        let camera_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            camera_buffer.as_entire_buffer_binding(),
                        ),
                    },
                    // wgpu::BindGroupEntry {
                    //     binding: 1,
                    //     resource: wgpu::BindingResource::Buffer(
                    //         projection_buffer.as_entire_buffer_binding(),
                    //     ),
                    // },
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
                    bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
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
                        buffers: &[model::ModelVertex::desc()],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: fragment_entry_point,
                        targets: &[Some(wgpu::ColorTargetState {
                            // 4.
                            format: context.config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
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

        debug!("{:?}", (bytemuck::cast_slice::<_, f32>(sprite.vertices())));
        debug!("{:?}", (bytemuck::cast_slice::<_, u16>(sprite.indices())));
        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(sprite.vertices()),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(sprite.indices()),
                usage: wgpu::BufferUsages::INDEX,
            });

        let num_indices = sprite.indices().len() as u32;

        Self {
            window,
            context,
            size,
            position: PhysicalPosition { x: 0.0, y: 0.0 },
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            sprite_sheet,
            sprite,
            camera,
            // projection,
            camera_bind_group,
        }
    }
    fn window(&self) -> &Window {
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
        }
    }

    fn set_position(&mut self, position: PhysicalPosition<f64>) {
        self.position = position
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }
    let mut state = State::new(&window).await;

    let _ = event_loop.run(move |event, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => state.resize(*physical_size),

                    WindowEvent::RedrawRequested => {
                        state.update();
                        match state.render() {
                            Ok(_) => {}
                            // Reconfigure the surface if lost
                            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                state.resize(state.size)
                            }
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }

                    WindowEvent::CursorMoved {
                        device_id: _,
                        position,
                    } => state.set_position(*position),
                    // TODO: refactor input controller
                    WindowEvent::KeyboardInput {
                        device_id: _,
                        event,
                        is_synthetic: _,
                    } => match event.state {
                        ElementState::Pressed => match event.physical_key {
                            PhysicalKey::Code(KeyCode::Space) => (),
                            _ => (),
                        },

                        ElementState::Released => (),
                    },

                    _ => {}
                }
            }
        }

        Event::AboutToWait => {
            // RedrawRequested will only trigger once unless we manually
            // request it.
            state.window().request_redraw();
        }
        _ => {}
    });
}
