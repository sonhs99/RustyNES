use alloc::vec::Vec;

use crate::{
    memory::{MemoryRead, MemoryWrite},
    ppu::{Tile, TileSize},
};

use super::{Cartridge, Mirroring, PRG_ROM_BANK_SIZE};

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub trainer: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub mirroring: Mirroring,
    pub writable: bool,
}

impl Rom {
    pub fn new(
        prg_rom: Vec<u8>,
        chr_rom: Vec<u8>,
        trainer: Vec<u8>,
        prg_ram: Vec<u8>,
        mirroring: Mirroring,
        writable: bool,
    ) -> Self {
        Self {
            prg_rom,
            chr_rom,
            prg_ram,
            trainer,
            mirroring,
            writable,
        }
    }
}

impl Cartridge for Rom {
    fn memory_read(&self, address: u16) -> MemoryRead {
        match address {
            0x6000..0x8000 => {
                if self.prg_ram.len() != 0 {
                    MemoryRead::Value(self.prg_ram[(address - 0x6000) as usize])
                } else {
                    MemoryRead::Value(0)
                }
            }
            0x8000..0xC000 => MemoryRead::Value(self.prg_rom[(address - 0x8000) as usize]),
            0xC000..=0xFFFF => {
                if self.prg_rom.len() <= PRG_ROM_BANK_SIZE {
                    MemoryRead::Value(self.prg_rom[(address - 0xC000) as usize])
                } else {
                    MemoryRead::Value(self.prg_rom[(address - 0x8000) as usize])
                }
            }
            _ => MemoryRead::Pass,
        }
    }

    fn memory_write(&mut self, address: u16, value: u8) -> MemoryWrite {
        match address {
            0x6000..0x8000 => {
                if self.prg_ram.len() != 0 {
                    self.prg_ram[(address - 0x6000) as usize] = value;
                } else if self.writable {
                    self.chr_rom[(address - 0x6000) as usize] = value;
                } else {
                    return MemoryWrite::Block;
                }
                MemoryWrite::Value(value)
            }
            _ => MemoryWrite::Block,
        }
    }
    fn tile(&self, idx: usize, size: TileSize) -> Tile {
        match size {
            TileSize::Tile8 => Tile::Tile8(self.chr_rom[idx..idx + 16].to_vec()),
            TileSize::Tile16 => Tile::Tile16(self.chr_rom[idx..idx + 32].to_vec()),
        }
    }

    fn ppu_read(&self, address: u16) -> MemoryRead {
        match address {
            0..0x2000 => MemoryRead::Value(self.chr_rom[address as usize]),
            _ => MemoryRead::Pass,
        }
    }

    fn ppu_write(&mut self, address: u16, value: u8) -> MemoryWrite {
        match address {
            0..0x2000 => {
                if self.writable {
                    self.chr_rom[address as usize] = value;
                    MemoryWrite::Value(value)
                } else {
                    MemoryWrite::Block
                }
            }
            _ => MemoryWrite::Block,
        }
    }
}
