mod mmc1;
mod nrom;
mod uxrom;

use alloc::vec::Vec;
use alloc::{boxed::Box, vec};
use libc_print::libc_println;

use crate::{
    device::IOHandler,
    memory::{MemoryBus, MemoryRead, MemoryWrite},
    ppu::{PpuHandler, Tile, TileSize},
};

const MAGIC_WORD: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_BANK_SIZE: usize = 0x4000;
const CHR_ROM_BANK_SIZE: usize = 0x2000;
const PRG_RAM_BANK_SIZE: usize = 0x2000;

pub trait Cartridge {
    fn memory_read(&self, address: u16) -> MemoryRead;
    fn memory_write(&mut self, address: u16, value: u8) -> MemoryWrite;
    fn tile(&self, idx: usize, size: TileSize) -> Tile;
    fn ppu_read(&self, address: u16) -> MemoryRead;
    fn ppu_write(&mut self, address: u16, value: u8) -> MemoryWrite;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

pub struct RomInfo {
    pub mapper: u8,
    pub mirroring: Mirroring,
    pub prg_rom_size: usize,
    pub chr_rom_size: usize,
}

pub struct Rom(Box<dyn Cartridge>, RomInfo);

impl Rom {
    pub fn new(raw: &Vec<u8>) -> Result<Self, ()> {
        if &raw[0..4] != MAGIC_WORD {
            return Err(());
        }
        let ctrl1 = raw[6];
        let ctrl2 = raw[7];

        // mapper
        let mapper = ctrl1 >> 4 | ctrl2 & 0b1111_0000;
        if ctrl2 & 0b0000_1100 != 0 {
            return Err(());
        }

        // mirroring
        let four_screen = ctrl1 & 0b0000_1000 != 0;
        let screen_direction = ctrl1 & 0b0000_0001 != 0;
        let mirroring = match (four_screen, screen_direction) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        // trainer
        let trainer_enable = ctrl1 & 0b0000_0100 != 0;
        let trainer = if trainer_enable {
            raw[0x10..0x210].to_vec()
        } else {
            Vec::new()
        };

        // prg_ram
        let prg_ram = if ctrl1 & 0b0000_0010 != 0 {
            Vec::<u8>::with_capacity((raw[8] as usize) * PRG_RAM_BANK_SIZE)
        } else {
            Vec::new()
        };

        // prg_rom & chr_rom
        let prg_rom_size = (raw[4] as usize) * PRG_ROM_BANK_SIZE;
        let chr_rom_size = (raw[5] as usize) * CHR_ROM_BANK_SIZE;

        let prg_rom_start = 0x10 + if trainer_enable { 0x200 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        let prg_rom = raw[prg_rom_start..prg_rom_start + prg_rom_size].to_vec();
        let chr_ram = chr_rom_size == 0;
        let chr_rom = if chr_ram {
            vec![0u8; 0x2000]
        } else {
            raw[chr_rom_start..chr_rom_start + chr_rom_size].to_vec()
        };

        // libc_println!("prg_rom size: 0x{:04X}", chr_rom_start - prg_rom_start);
        // libc_println!(
        //     "chr_rom size: 0x{:04X}",
        //     raw[chr_rom_start..chr_rom_start + chr_rom_size].len()
        // );

        // libc_println!(
        //     "Entry Point : {:02X} {:02X}",
        //     raw[prg_rom_start + prg_rom_size - 4],
        //     raw[prg_rom_start + prg_rom_size - 3]
        // );

        // libc_println!(
        //     "NMI INT Vector : {:02X} {:02X}",
        //     raw[prg_rom_start + prg_rom_size - 6],
        //     raw[prg_rom_start + prg_rom_size - 5]
        // );

        let info = RomInfo {
            mapper,
            mirroring,
            prg_rom_size,
            chr_rom_size,
        };

        match mapper {
            0 => {
                use nrom::Rom;
                Ok(Self(
                    Box::new(Rom::new(
                        prg_rom, chr_rom, trainer, prg_ram, mirroring, chr_ram,
                    )),
                    info,
                ))
            }
            2 => {
                use uxrom::Rom;
                Ok(Self(
                    Box::new(Rom::new(
                        prg_rom, chr_rom, trainer, prg_ram, mirroring, chr_ram,
                    )),
                    info,
                ))
            }
            _ => Err(()),
        }
    }

    pub fn info<'a>(&'a self) -> &'a RomInfo {
        &self.1
    }
}

impl IOHandler for Rom {
    fn read(&mut self, mmu: &MemoryBus, address: u16) -> MemoryRead {
        self.0.memory_read(address)
    }

    fn write(&mut self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite {
        self.0.memory_write(address, value)
    }
}

impl PpuHandler for Rom {
    fn tile(&self, idx: usize, size: TileSize) -> Tile {
        self.0.tile(idx, size)
    }

    fn read(&self, address: u16) -> MemoryRead {
        self.0.ppu_read(address)
    }

    fn write(&mut self, address: u16, value: u8) -> MemoryWrite {
        self.0.ppu_write(address, value)
    }
}
