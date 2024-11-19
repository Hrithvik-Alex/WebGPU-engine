use crate::model;

pub trait Component {
    fn name(&self) -> String;
}

pub struct VertexArrayComponent {
    pub vertices: Vec<cgmath::Vector2<f32>>,
    pub indices: Vec<u32>,
    pub tex_coords: Vec<cgmath::Vector2<f32>>,
}

impl Component for VertexArrayComponent {
    fn name(&self) -> String {
        return "VertexArray".to_string();
    }
}

pub struct PositionComponent {
    pub position: cgmath::Vector2<f32>,
    pub scale: f32,
}

impl Component for PositionComponent {
    fn name(&self) -> String {
        return "Position".to_string();
    }
}
