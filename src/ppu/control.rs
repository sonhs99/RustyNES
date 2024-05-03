use bitflags::bitflags;

use crate::cartridge::Mirroring;

use super::TileSize;

bitflags! {
    pub struct ControllRegister: u8 {
        const NAMETABLE1                 = 0b0000_0001;
        const NAMETABLE2                 = 0b0000_0010;
        const VRAM_ADD_INCREMENT         = 0b0000_0100;
        const SPRITE_PATTERN_ADDR        = 0b0000_1000;
        const BACKGROUND_PATTERN_ADDR    = 0b0001_0000;
        const SPRITE_SIZE                = 0b0010_0000;
        const MASTER_SLAVE_SELECT        = 0b0100_0000;
        const GENERATE_NMI               = 0b1000_0000;
    }
}

impl ControllRegister {
    pub fn new() -> Self {
        ControllRegister::from_bits_truncate(0x00)
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if !self.contains(ControllRegister::VRAM_ADD_INCREMENT) {
            1
        } else {
            32
        }
    }

    pub fn sprite_pattern_addr(&self) -> u16 {
        if !self.contains(ControllRegister::SPRITE_PATTERN_ADDR) {
            0
        } else {
            0x1000
        }
    }

    pub fn background_pattern_addr(&self) -> u16 {
        if !self.contains(ControllRegister::BACKGROUND_PATTERN_ADDR) {
            0
        } else {
            0x1000
        }
    }

    pub fn sprite_size(&self) -> TileSize {
        if !self.contains(ControllRegister::SPRITE_SIZE) {
            TileSize::Tile8
        } else {
            TileSize::Tile16
        }
    }

    pub fn master_slave_select(&self) -> u8 {
        if !self.contains(ControllRegister::MASTER_SLAVE_SELECT) {
            0
        } else {
            1
        }
    }

    pub fn nametable(&self, mirroring: Mirroring) -> u16 {
        (match (mirroring, self.bits() & 0x03) {
            (Mirroring::Vertical, 0)
            | (Mirroring::Vertical, 2)
            | (Mirroring::Horizontal, 0)
            | (Mirroring::Horizontal, 1) => 0,
            (Mirroring::Vertical, 1)
            | (Mirroring::Vertical, 3)
            | (Mirroring::Horizontal, 2)
            | (Mirroring::Horizontal, 3) => 1,
            (_, nametable) => nametable,
        }) as u16
            * 0x400
    }

    pub fn generate_vblank_nmi(&self) -> bool {
        self.contains(ControllRegister::GENERATE_NMI)
    }

    pub fn update(&mut self, data: u8) {
        *self = ControllRegister::from_bits_truncate(data);
    }
}
