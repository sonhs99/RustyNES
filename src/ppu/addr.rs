pub struct AddressRegister {
    value: u16,
    hi_ptr: bool,
}

impl AddressRegister {
    pub fn new() -> Self {
        Self {
            value: 0,
            hi_ptr: true,
        }
    }

    pub fn set(&mut self, data: u16) {
        self.value = data
    }

    pub fn update(&mut self, data: u8) {
        let data = data as u16;
        if self.hi_ptr {
            self.value = data << 8 | (self.value & 0xFF);
        } else {
            self.value = data | (self.value & 0xFF00);
        }

        self.value &= 0x3FFF;

        self.hi_ptr = !self.hi_ptr;
    }

    pub fn increment(&mut self, inc: u8) {
        self.value = self.value.wrapping_add(inc as u16);
        self.value &= 0x3FFF;
    }

    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }

    pub fn get(&self) -> u16 {
        self.value
    }
}
