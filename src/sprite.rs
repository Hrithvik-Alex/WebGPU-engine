use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::component;
use crate::component::EntityMap;
use crate::context;
use crate::texture;

use cgmath::Vector2;
use log::debug;
// pub struct Sprite {
//     sprite_sheet: Arc<SpriteSheet>,
//     position: Vector2<f32>,
//     pub sheet_position: Vector2<u32>,
//     scale: f32,
// }

// impl Sprite {
//     pub fn new(sprite_sheet: Arc<SpriteSheet>, scale: f32, position: Vector2<f32>) -> Self {
//         Self {
//             sprite_sheet,
//             position,
//             sheet_position: Vector2::new(0, 0),
//             scale,
//         }
//     }

//     pub fn bind_group(&self) -> &wgpu::BindGroup {
//         &self.sprite_sheet.texture.bind_group
//     }

//     pub fn vertices(&self) -> [model::ModelVertex2d; 4] {
//         let x = self.position.x;
//         let y = self.position.y;

//         let sheet_x = self.sheet_position.x as f32 * self.sprite_sheet.sprite_width as f32
//             / self.sprite_sheet.dimensions.0 as f32;
//         let sheet_y = self.sheet_position.y as f32 * self.sprite_sheet.sprite_height as f32
//             / self.sprite_sheet.dimensions.1 as f32;

//         [
//             // Changed
//             model::ModelVertex2d {
//                 // TOP-LEFT
//                 position: [x, y + self.scale],
//                 tex_coords: [sheet_x, sheet_y],
//                 normal: [0.0, 0.0, 0.0],
//             },
//             model::ModelVertex2d {
//                 // TOP-RIGHT
//                 position: [x + self.scale, y + self.scale],
//                 tex_coords: [
//                     sheet_x
//                         + self.sprite_sheet.sprite_width as f32
//                             / self.sprite_sheet.dimensions.0 as f32,
//                     sheet_y,
//                 ],
//                 normal: [0.0, 0.0, 0.0],
//             },
//             model::ModelVertex2d {
//                 // BOTTOM-LEFT
//                 position: [x, y],
//                 tex_coords: [
//                     sheet_x,
//                     sheet_y
//                         + self.sprite_sheet.sprite_height as f32
//                             / self.sprite_sheet.dimensions.1 as f32,
//                 ],
//                 normal: [0.0, 0.0, 0.0],
//             },
//             model::ModelVertex2d {
//                 // BOTTOM-RIGHT
//                 position: [x + self.scale, y],
//                 tex_coords: [
//                     sheet_x
//                         + (self.sprite_sheet.sprite_width as f32)
//                             / self.sprite_sheet.dimensions.0 as f32,
//                     sheet_y
//                         + self.sprite_sheet.sprite_height as f32
//                             / self.sprite_sheet.dimensions.1 as f32,
//                 ],
//                 normal: [0.0, 0.0, 0.0],
//             },
//         ]
//     }

//     pub fn indices(&self) -> [u16; 6] {
//         [0, 2, 3, 0, 3, 1]
//     }
//     pub fn get_position(&self) -> Vector2<f32> {
//         self.position
//     }
//     pub fn update_position(&mut self, position: Vector2<f32>) {
//         self.position = position;
//     }

//     pub fn update_sheet_position(&mut self, sheet_index: u32) {
//         let sheet_position = self.sprite_sheet.get_position_by_index(sheet_index);
//         self.sheet_position = sheet_position;
//     }
// }

pub struct SheetPositionComponent {
    pub sprite_sheet: Rc<RefCell<SpriteSheet>>,
    pub sheet_position: cgmath::Vector2<u32>,
}

impl component::Component for SheetPositionComponent {
    fn name(&self) -> String {
        "SheetPosition".to_string()
    }
}

pub struct SpriteSheetSystem {}

impl SpriteSheetSystem {
    pub fn update(
        vertex_array_components: &mut EntityMap<component::VertexArrayComponent>,
        sheet_position_components: &EntityMap<SheetPositionComponent>,
    ) {
        vertex_array_components
            .iter_mut()
            .zip(sheet_position_components.iter())
            .for_each(
                |((_, vertex_array_component), (_, sheet_position_component))| {
                    if let (Some(vertex_array_component), Some(sheet_position_component)) =
                        (vertex_array_component, sheet_position_component)
                    {
                        sheet_position_component
                            .sprite_sheet
                            .borrow()
                            .adjust_tex_coords(
                                vertex_array_component,
                                sheet_position_component.sheet_position,
                            )
                    }
                },
            );
    }
}

pub struct SpriteSheet {
    sprite_width: u32,
    sprite_height: u32,
    dimensions: (u32, u32),
    texture: Arc<texture::Texture>,

    texture_path: String,
    normal_path: Option<String>,
    manual_premultiply: bool,
}

impl SpriteSheet {
    pub fn new(
        context: &context::Context,
        texture_path: String,
        normal_path: Option<String>,
        sprite_width: u32,
        sprite_height: u32,
        manual_premultiply: bool,
    ) -> Self {
        let texture = Arc::new(
            crate::texture::Texture::from_path(
                &context.device,
                &context.queue,
                texture_path.clone(),
                normal_path.clone(),
                manual_premultiply,
            )
            .unwrap(),
        );

        let dimensions = texture.dimensions;

        debug!("{:?}", dimensions);
        Self {
            sprite_width,
            sprite_height,
            dimensions,
            texture,
            texture_path,
            normal_path,
            manual_premultiply,
        }
    }

    pub fn get_position_by_index(&self, index: u32) -> Vector2<u32> {
        let num_sprites = self.dimensions.0 / self.sprite_width;
        Vector2::new(index % num_sprites, index / num_sprites)
    }

    pub fn texture(&self) -> Arc<texture::Texture> {
        return self.texture.clone();
    }

    pub fn resize(&mut self, context: &context::Context) {
        self.texture = Arc::new(
            crate::texture::Texture::from_path(
                &context.device,
                &context.queue,
                self.texture_path.clone(),
                self.normal_path.clone(),
                self.manual_premultiply,
            )
            .unwrap(),
        )
    }

    pub fn adjust_tex_coords(
        &self,
        vertex_array: &mut component::VertexArrayComponent,
        sheet_position: Vector2<u32>,
    ) {
        let step_x = self.sprite_width as f32 / self.dimensions.0 as f32;
        let step_y = self.sprite_height as f32 / self.dimensions.1 as f32;
        let sheet_x = sheet_position.x as f32 * step_x;
        let sheet_y = sheet_position.y as f32 * step_y;

        // debug!(
        //     "{:?} {:?} {:?} {:?} {:?}",
        //     sheet_x, sheet_y, sheet_position, step_x, step_y
        // );
        vertex_array.tex_coords = vertex_array
            .whole_tex_coords
            .iter()
            .map(|whole_tex_coord| {
                cgmath::Vector2::new(
                    sheet_x + whole_tex_coord.x * step_x,
                    sheet_y + whole_tex_coord.y * step_y,
                )
            })
            .collect();

        // debug!("{:?}", vertex_array.tex_coords);
        // debug!("{:?}", sheet_position);
    }
}
