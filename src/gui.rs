use std::sync::Arc;

use egui::epaint::Shadow;
use egui::{
    include_image, Align2, ColorImage, Context, FontId, RichText, TextureHandle, TextureOptions,
    Vec2, Visuals,
};
use egui_wgpu::Renderer;
use egui_wgpu::ScreenDescriptor;

use egui_wgpu::wgpu;
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};
use egui_winit::winit::event::WindowEvent;
use egui_winit::winit::window::Window;
use egui_winit::State;
use log::debug;

use crate::{context, game};

pub struct GuiInfo {
    pub fps: u32,
    pub notes_collected: u32,
}

pub struct Gui {
    pub context: Context,
    pub state: State,
    renderer: Renderer,
    scroll_image: TextureHandle,
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
        let rgba = image::load_from_memory(
            &std::fs::read("./assets/scroll.png").expect("failed to load scroll"),
        )
        .unwrap()
        .to_rgba8()
        .to_vec();
        let image = ColorImage::from_rgba_unmultiplied([16, 16], &rgba);
        let scroll_image = egui_context.load_texture("scroll", image, TextureOptions::default());

        Self {
            context: egui_context,
            state: egui_state,
            renderer: egui_renderer,
            scroll_image,
        }
    }

    // pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) {
    //     let _ = self.state.on_window_event(window, event);
    // }

    pub fn draw(
        &mut self,
        context: &context::Context,
        encoder: &mut CommandEncoder,
        window: Arc<Window>,
        window_surface_view: &TextureView,
        info: &GuiInfo,
        game_mode: &mut game::GameMode, // mut run_ui: impl FnMut(&Context),
    ) {
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [context.config.width, context.config.height],
            pixels_per_point: window.scale_factor() as f32,
        };
        let rect = self.context.screen_rect().center();

        // self.state.set_pixels_per_point(window.scale_factor() as f32);
        let raw_input = self.state.take_egui_input(&window);
        let full_output = self.context.run(raw_input, |ctx| {
            egui::Area::new(egui::Id::new("title"))
                .movable(false)
                .anchor(Align2::CENTER_TOP, [0.0, 10.0])
                .show(&ctx, |mut ui| ui.label("Halex"));
            // egui::Label::new("Halex").show(ctx, |ui| {
            //     ui.label("Halex")
            //     // .background_color(Color32::from_black_alpha(0));
            // });



            match *game_mode {
                game::GameMode::POPUP => {

                
                let popup_size = egui::vec2(rect.x * 0.6, rect.y * 0.9); // Desired popup size

                let mut open = true;
                egui::Window::new("Popup")
                    .collapsible(false)
                    .resizable(false)
                    .current_pos(rect - popup_size / 2.)
                    .default_size(popup_size)
                    .open(&mut open)
                    .show(ctx, |ui| {
                        ui.label("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer enim lacus, commodo quis sem quis, posuere lacinia sem. Quisque quis lacus posuere, egestas massa quis, semper dolor. Fusce euismod sagittis nisi sit amet commodo. Cras ultricies rhoncus tortor a varius. Nam dui quam, feugiat nec ultrices eget, dictum condimentum mi. Nam sit amet metus ultrices, ultricies orci eu, consequat ipsum. Duis in enim hendrerit, pretium sapien at, congue libero.

");

                    });
                
                if !open {
                    *game_mode = game::GameMode::STANDARD;
                }

            },
            game::GameMode::STANDARD => {
                egui::Area::new(egui::Id::new("collectible info"))
                .movable(false)
                .anchor(Align2::LEFT_BOTTOM, [10.0, -10.0])
                .show(&ctx, |mut ui| {



                    ui.horizontal(|ui| {
                        ui.add(
                            egui::Image::new((
                                self.scroll_image.id(),
                                self.scroll_image.size_vec2(),
                            ))
                            .fit_to_exact_size(Vec2 { x: 64., y: 64. })
                            .maintain_aspect_ratio(true),
                        );

                        ui.label(
                            RichText::new(format!("{}", info.notes_collected))
                                .font(FontId::proportional(40.0)),
                        )
                    })
                });
            }
        }

            egui::Window::new("statistics")
                // .vscroll(true)
                .default_open(true)
                .max_width(1000.0)
                .max_height(800.0)
                .default_width(800.0)
                .resizable(true)
                .anchor(Align2::LEFT_TOP, [0.0, 0.0])
                .show(&ctx, |mut ui| {
                    // if ui.add(egui::Button::new("Click me")).clicked() {
                    //     debug!("PRESSED")
                    // }

                    ui.label(format!("FPS: {}", info.fps));
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
