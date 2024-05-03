use alloc::vec::Vec;
use libc_print::libc_println;

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
    pub bank: usize,
    last_bank: usize,
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
        let last_bank = prg_rom.len() / PRG_ROM_BANK_SIZE - 1;
        Self {
            prg_rom,
            chr_rom,
            prg_ram,
            trainer,
            mirroring,
            writable,
            bank: 0,
            last_bank,
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
            0x8000..0xC000 => {
                let base = self.bank * PRG_ROM_BANK_SIZE;
                let offset = (address - 0x8000) as usize;
                MemoryRead::Value(self.prg_rom[base + offset])
            }
            0xC000..=0xFFFF => {
                let base = self.last_bank * PRG_ROM_BANK_SIZE;
                let offset = (address - 0xC000) as usize;
                MemoryRead::Value(self.prg_rom[base + offset])
            }
            _ => MemoryRead::Pass,
        }
    }

    fn memory_write(&mut self, address: u16, value: u8) -> MemoryWrite {
        match address {
            0x6000..0x8000 => {
                if self.prg_ram.len() != 0 {
                    self.prg_ram[(address - 0x6000) as usize] = value;
                } else {
                    self.chr_rom[(address - 0x6000) as usize] = value;
                }
                MemoryWrite::Value(value)
            }
            0x8000..=0xFFFF => {
                self.bank = (value & 0x0F) as usize;
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
