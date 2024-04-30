pub struct StatusRegister(u8);

impl StatusRegister {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn update(&mut self, data: u8) {
        self.0 = data;
    }

    pub fn get(&self) -> u8 {
        self.0
    }

    pub fn bus(&self) -> u8 {
        self.0 & 0b0001_1111
    }

    pub fn sprite_overflow(&self) -> bool {
        self.0 & 0b0010_0000 != 0
    }

    pub fn sprite_0_hit(&self) -> bool {
        self.0 & 0b0100_0000 != 0
    }

    pub fn vblank(&self) -> bool {
        self.0 & 0b1000_0000 != 0
    }

    pub fn set_vblank(&mut self, flag: bool) {
        self.0 = if flag {
            self.0 | 0b1000_0000
        } else {
            self.0 & 0b0111_1111
        }
    }

    pub fn set_sprite_0_hit(&mut self, flag: bool) {
        self.0 = if flag {
            self.0 | 0b0100_0000
        } else {
            self.0 & 0b1011_1111
        }
    }
}
