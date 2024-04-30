pub struct ScrollRegister {
    pub x: u8,
    pub y: u8,
    toggle: bool,
}

impl ScrollRegister {
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            toggle: false,
        }
    }

    pub fn update(&mut self, value: u8) {
        if !self.toggle {
            self.x = value;
        } else {
            self.y = value;
        }
        self.toggle = !self.toggle;
    }

    pub fn reset_latch(&mut self) {
        self.toggle = false;
    }
}
