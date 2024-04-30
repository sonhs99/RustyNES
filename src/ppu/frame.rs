use alloc::vec::Vec;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

pub struct Frame {
    pub data: [u8; WIDTH * HEIGHT],
}

impl Frame {
    pub fn new() -> Self {
        Self {
            data: [0; WIDTH * HEIGHT],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: u8) {
        let base = y * WIDTH + x;
        if base < self.data.len() {
            self.data[base] = color;
        }
    }
}
