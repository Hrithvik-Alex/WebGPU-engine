use crate::component;
use crate::context;
use crate::model;
use crate::model::ModelVertex2d;

use wgpu::util::DeviceExt;

pub struct RenderSystem<'a> {
    positions: Vec<&'a component::PositionComponent>,
    vertex_arrays: Vec<&'a component::VertexArrayComponent>,
}

impl<'a> RenderSystem<'a> {
    pub fn new(
        positions: Vec<&'a component::PositionComponent>,
        vertex_arrays: Vec<&'a component::VertexArrayComponent>,
    ) -> Self {
        assert!(positions.len() == vertex_arrays.len());
        Self {
            positions,
            vertex_arrays,
        }
    }
    pub fn render(&self, context: &context::Context) -> Result<(), wgpu::SurfaceError> {
        let model_vertices: Vec<model::ModelVertex2d> = self
            .positions
            .iter()
            .zip(self.vertex_arrays.iter())
            .map(|(&pos, &vertex_array)| {
                vertex_array
                    .vertices
                    .iter()
                    .zip(vertex_array.tex_coords.iter())
                    .map(|(vertex_pos, &tex_coord)| model::ModelVertex2d {
                        position: ((vertex_pos * pos.scale) + pos.position).into(),
                        tex_coords: tex_coord.into(),
                        normal: [0.0, 0.0, 0.0],
                    })
            })
            .flatten()
            .collect();
        let mut all_vertices: Vec<ModelVertex2d> = vec![];
        let mut all_indices: Vec<u32> = vec![];
        for i in 0..self.positions.len() {
            let vertex_array = self.vertex_arrays[i];
            let pos = self.positions[i];
            let model_vertices: Vec<ModelVertex2d> = vertex_array
                .vertices
                .iter()
                .zip(vertex_array.tex_coords.iter())
                .map(|(vertex_pos, &tex_coord)| model::ModelVertex2d {
                    position: ((vertex_pos * pos.scale) + pos.position).into(),
                    tex_coords: tex_coord.into(),
                    normal: [0.0, 0.0, 0.0],
                })
                .collect();

            all_vertices.extend_from_slice(&model_vertices);
            all_indices.extend_from_slice(&vertex_array.indices);
        }

        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&model_vertices),
                usage: wgpu::BufferUsages::VERTEX,
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

        context.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
