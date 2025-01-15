use crate::{
    component::{self, EntityMap},
    state,
};

pub struct MiraGameState {
    pub notes_collected: u32,
}

impl MiraGameState {
    pub fn new() -> Self {
        Self { notes_collected: 0 }
    }

    pub fn reset(&mut self) {}
}

#[derive(Clone, Copy, PartialEq)]
pub enum GameMode {
    STANDARD,
    POPUP,
}
