use std::sync::Arc;

use egui::epaint::Shadow;
use egui::{Align2, Context, Visuals};
use egui_wgpu::Renderer;
use egui_wgpu::ScreenDescriptor;

use egui_wgpu::wgpu;
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};
use egui_winit::winit::event::WindowEvent;
use egui_winit::winit::window::Window;
use egui_winit::State;

use crate::context;

pub struct Gui {
    pub context: Context,
    state: State,
    renderer: Renderer,
}

impl Gui {
    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        window: Arc<Window>,
    ) -> Self {
        let egui_context = Context::default();
        let id = egui_context.viewport_id();

        const BORDER_RADIUS: f32 = 2.0;

        let visuals = Visuals {
            window_rounding: egui::Rounding::same(BORDER_RADIUS),
            window_shadow: Shadow::NONE,
            // menu_rounding: todo!(),
            ..Default::default()
        };

        egui_context.set_visuals(visuals);

        let egui_state = State::new(egui_context.clone(), id, &window, None, None, None);

        // egui_state.set_pixels_per_point(window.scale_factor() as f32);
        let egui_renderer = Renderer::new(
            device,
            output_color_format,
            output_depth_format,
            msaa_samples,
            false,
        );

        Self {
            context: egui_context,
            state: egui_state,
            renderer: egui_renderer,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    pub fn draw(
        &mut self,
        context: &context::Context,
        encoder: &mut CommandEncoder,
        window: Arc<Window>,
        window_surface_view: &TextureView,
        // mut run_ui: impl FnMut(&Context),
    ) {
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [context.config.width, context.config.height],
            pixels_per_point: window.scale_factor() as f32,
        };
        // self.state.set_pixels_per_point(window.scale_factor() as f32);
        let raw_input = self.state.take_egui_input(&window);
        let full_output = self.context.run(raw_input, |ui| {
            egui::Window::new("Streamline CFD")
                // .vscroll(true)
                .default_open(true)
                .max_width(1000.0)
                .max_height(800.0)
                .default_width(800.0)
                .resizable(true)
                .anchor(Align2::LEFT_TOP, [0.0, 0.0])
                .show(&ui, |mut ui| {
                    if ui.add(egui::Button::new("Click me")).clicked() {
                        println!("PRESSED")
                    }

                    ui.label("Slider");
                    // ui.add(egui::Slider::new(_, 0..=120).text("age"));
                    ui.end_row();

                    // proto_scene.egui(ui);
                });
        });

        self.state
            .handle_platform_output(&window, full_output.platform_output);

        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(&context.device, &context.queue, *id, &image_delta);
        }
        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &tris,
            &screen_descriptor,
        );
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &window_surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                label: Some("egui main render pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.renderer
                .render(&mut rpass.forget_lifetime(), &tris, &screen_descriptor);
            // drop(rpass);
        }
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
}
