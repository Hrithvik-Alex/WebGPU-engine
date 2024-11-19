use crate::component::*;
use std::collections::HashMap;

pub struct Entity {
    components: HashMap<String, Box<dyn Component>>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn add_component(&mut self, component: Box<dyn Component>) {
        if self.components.contains_key(&component.name()) {
            panic!("component {:?} already exists", component.name())
        } else {
            self.components.insert(component.name(), component);
        }
    }
}
