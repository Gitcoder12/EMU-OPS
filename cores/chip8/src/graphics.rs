use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Graphics {
    pub display: [[bool; 64]; 32],
    pub needs_render: bool,
}

impl Graphics {
    pub fn new() -> Self {
        Self {
            display: [[false; 64]; 32],
            needs_render: true,
        }
    }

    pub fn clear(&mut self) {
        for row in self.display.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = false;
            }
        }
        self.needs_render = true;
    }
}
