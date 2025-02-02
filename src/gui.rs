use std::sync::Arc;

use bytemuck::Zeroable;
use cgmath::num_traits::{clamp, clamp_max, clamp_min};
use egui::epaint::Shadow;
use egui::{
    Align2, Color32, ColorImage, Context, FontId, Pos2, Rect, RichText, Rounding, Stroke,
    TextureHandle, TextureOptions, Vec2, Visuals,
};
use egui_wgpu::Renderer;
use egui_wgpu::ScreenDescriptor;

use egui_wgpu::wgpu;
use egui_wgpu::wgpu::{CommandEncoder, Device, TextureFormat, TextureView};
use egui_winit::winit::window::Window;
use egui_winit::State;

use crate::{context, game};

#[derive(Clone, Copy)]
pub enum PopupType {
    SCROLL,
    WOOD,
}

pub struct GuiInfo {
    pub fps: u32,
    pub notes_collected: u32,
    pub popup_text: &'static str,
    pub popup_type: PopupType,
}

pub struct Gui {
    pub context: Context,
    pub state: State,
    renderer: Renderer,
    scroll_image: TextureHandle,
    scroll_background_image: TextureHandle,
    wood_background_image: TextureHandle,
    scroll_offset: egui::Vec2,
    scroll_content_size: Option<egui::Vec2>,
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
        let scroll_image = {
            let rgba = image::load_from_memory(include_bytes!("../assets/scroll.png"))
                .unwrap()
                .to_rgba8()
                .to_vec();
            let image = ColorImage::from_rgba_unmultiplied([16, 16], &rgba);
            egui_context.load_texture("scroll", image, TextureOptions::default())
        };

        let scroll_background_image = {
            let rgba = image::load_from_memory(include_bytes!("../assets/Pergament9.png"))
                .unwrap()
                .to_rgba8()
                .to_vec();
            let image = ColorImage::from_rgba_unmultiplied([1160, 965], &rgba);
            egui_context.load_texture("scroll_background", image, TextureOptions::default())
        };

        let wood_background_image = {
            let rgba = image::load_from_memory(include_bytes!("../assets/wood_bg.png"))
                .unwrap()
                .to_rgba8()
                .to_vec();
            let image = ColorImage::from_rgba_unmultiplied([879, 879], &rgba);
            egui_context.load_texture("wood_background", image, TextureOptions::default())
        };

        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Geo-Regular".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/Geo-Regular.ttf")),
        );

        fonts.families.insert(
            egui::FontFamily::Name("Geo-Regular".into()),
            vec!["Geo-Regular".to_owned()],
        );

        egui_context.set_fonts(fonts);

        Self {
            context: egui_context,
            state: egui_state,
            renderer: egui_renderer,
            scroll_image,
            scroll_background_image,
            wood_background_image,
            scroll_offset: egui::Vec2::zeroed(),
            scroll_content_size: None,
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
            // egui::Area::new(egui::Id::new("title"))
            //     .movable(false)
            //     .anchor(Align2::CENTER_TOP, [0.0, 10.0])
            //     .show(&ctx, |mut ui| ui.label("Halex"));

            let scrolll = egui::Image::new((self.scroll_image.id(), self.scroll_image.size_vec2()))
                .fit_to_exact_size(Vec2 { x: 64., y: 64. })
                .maintain_aspect_ratio(true);

            match *game_mode {
                game::GameMode::POPUP => {
                    let popup_size = egui::vec2(rect.x, rect.y); // Desired popup size

                    let scroll_top_margin = clamp_min(30. - self.scroll_offset.y, 5.);
                    let scroll_bot_margin = clamp(
                        self.scroll_offset.y + rect.y
                            - self
                                .scroll_content_size
                                .map_or_else(|| rect.y, |content_size| content_size.y),
                        0.,
                        30.,
                    );

                    // .fit_to_exact_size(popup_size);
                    // .maintain_aspect_ratio(true);
                    // let original_visuals = Visuals::default();
                    // \self.context.set_visuals(visuals); // Store the original visuals

                    egui::Window::new("Popup")
                        .collapsible(false)
                        .resizable(false)
                        .movable(false)
                        .current_pos(rect - popup_size / 2.)
                        .default_size(popup_size)
                        .title_bar(false)
                        .frame(egui::Frame {
                            rounding: egui::Rounding::same(0.),
                            shadow: Shadow::NONE,
                            fill: Color32::from_black_alpha(0),
                            stroke: Stroke::NONE,

                            ..Default::default()
                        })
                        .show(ctx, |ui| {
                            ui.visuals_mut().override_text_color = Some(Color32::BLACK);

                            match info.popup_type {
                                PopupType::SCROLL => {
                                    let scroll_background = egui::Image::new((
                                        self.scroll_background_image.id(),
                                        self.scroll_background_image.size_vec2(),
                                    ))
                                    .uv(self.scroll_content_size.map_or_else(
                                        || Rect {
                                            min: Pos2 { x: 0.0, y: 0.0 },
                                            max: Pos2 { x: 1.0, y: 1.0 },
                                        },
                                        |content_size| Rect {
                                            min: Pos2 {
                                                x: 0.0,
                                                y: self.scroll_offset.y / content_size.y,
                                            },
                                            max: Pos2 {
                                                x: 1.0,
                                                y: clamp_max(
                                                    (self.scroll_offset.y + rect.y
                                                        - scroll_top_margin
                                                        - scroll_bot_margin)
                                                        / content_size.y,
                                                    1.,
                                                ),
                                            },
                                        },
                                    ));

                                    scroll_background.paint_at(
                                        ui,
                                        egui::Rect {
                                            min: rect - popup_size / 2.,
                                            max: rect + popup_size / 2.,
                                        },
                                    )
                                }
                                PopupType::WOOD => {
                                    let wood_background = egui::Image::new((
                                        self.wood_background_image.id(),
                                        self.wood_background_image.size_vec2(),
                                    ));
                                    wood_background.paint_at(
                                        ui,
                                        egui::Rect {
                                            min: rect - popup_size / 2.,
                                            max: rect + popup_size / 2.,
                                        },
                                    );

                                    ui.painter().rect_filled(
                                        egui::Rect {
                                            min: rect - popup_size / 2. * 0.9,
                                            max: rect + popup_size / 2. * 0.9,
                                        },
                                        Rounding::ZERO,
                                        Color32::from_rgb(255, 253, 208),
                                    );
                                }
                            };

                            egui::Frame::group(ui.style())
                                .inner_margin(egui::Margin {
                                    left: rect.x * 0.06,
                                    right: rect.x * 0.06,
                                    top: scroll_top_margin,
                                    bottom: scroll_bot_margin,
                                })
                                .stroke(Stroke::NONE)
                                .show(ui, |ui| {
                                    let scroll_area = egui::ScrollArea::vertical()
                                        .scroll_bar_visibility(
                                            egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                                        );

                                    let output = scroll_area.show(ui, |ui| {
                                        ui.vertical_centered(|ui| {
                                            ui.label(RichText::new(info.popup_text).font(FontId {
                                                size: 25.0,
                                                family: egui::epaint::FontFamily::Name(
                                                    "Geo-Regular".into(),
                                                ),
                                            }))
                                        })
                                    });

                                    self.scroll_offset = output.state.offset;
                                    self.scroll_content_size = Some(output.content_size);
                                });

                            // ui.painter().image(texture_id, rect, uv, tint)
                        });

                    egui::Area::new(egui::Id::new("popup controls"))
                        .movable(false)
                        .anchor(Align2::RIGHT_TOP, [-10.0, 10.0])
                        .show(&ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("ESC");
                                ui.label("to close");
                            })
                        });

                    // self.context.set_visuals(original_visuals); // Restore the original visuals after the window
                }
                game::GameMode::STANDARD => {
                    egui::Area::new(egui::Id::new("collectible info"))
                        .movable(false)
                        .anchor(Align2::LEFT_BOTTOM, [10.0, -10.0])
                        .show(&ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.add(scrolll);

                                ui.label(
                                    RichText::new(format!("{}", info.notes_collected))
                                        .font(FontId::proportional(40.0)),
                                )
                            })
                        });
                }
            }

            // egui::Window::new("statistics")
            //     // .vscroll(true)
            //     .default_open(true)
            //     .max_width(1000.0)
            //     .max_height(800.0)
            //     .default_width(800.0)
            //     .resizable(true)
            //     .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            //     .show(&ctx, |mut ui| {
            //         ui.label(format!("FPS: {}", info.fps));
            //         ui.end_row();

            //     });
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
