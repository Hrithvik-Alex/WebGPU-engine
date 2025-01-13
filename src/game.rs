use crate::state;

pub struct MiraGameState {
    pub notes_collected: u32,
}

impl MiraGameState {
    fn new() -> Self {
        Self { notes_collected: 0 }
    }

    fn reset(&self) {}

    fn update(&self, state: &state::State) {
        if (true) {
            self.notes_collected += 1;
        }
    }
}
