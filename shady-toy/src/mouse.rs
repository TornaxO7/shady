use winit::{dpi::PhysicalPosition, event::ElementState};

#[derive(Debug)]
pub struct Mouse {
    pos: PhysicalPosition<f32>,

    prev_state: ElementState,
    curr_state: ElementState,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            pos: PhysicalPosition::default(),

            prev_state: ElementState::Released,
            curr_state: ElementState::Released,
        }
    }

    pub fn mouse_input(&mut self, state: ElementState) {
        self.prev_state = self.curr_state;
        self.curr_state = state;
    }

    pub fn cursor_moved(&mut self, pos: PhysicalPosition<f64>) {
        self.pos = PhysicalPosition {
            x: pos.x as f32,
            y: pos.y as f32,
        }
    }

    pub fn pos(&self) -> PhysicalPosition<f32> {
        self.pos
    }

    pub fn is_pressed(&self) -> bool {
        self.curr_state.is_pressed()
    }

    pub fn was_pressed(&self) -> bool {
        self.prev_state.is_pressed()
    }
}
