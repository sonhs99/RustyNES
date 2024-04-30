use bitflags::bitflags;

bitflags! {
    pub struct MaskRegister: u8 {
        const GRAYSCALE          = 0b0000_0001;
        const BACKGROUND_LEFT    = 0b0000_0010;
        const SPRITE_LEFT        = 0b0000_0100;
        const BACKGROUND         = 0b0000_1000;
        const SPRITE             = 0b0001_0000;
        const EMPHASIZE_RED      = 0b0010_0000;
        const EMPHASIZE_GREEN    = 0b0100_0000;
        const EMPHASIZE_BLUE     = 0b1000_0000;
    }
}

impl MaskRegister {
    pub fn new() -> Self {
        MaskRegister::from_bits_truncate(0x00)
    }

    pub fn update(&mut self, data: u8) {
        *self = MaskRegister::from_bits_truncate(data);
    }
}
