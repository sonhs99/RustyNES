use alloc::vec;
use alloc::vec::Vec;
use libc_print::libc_println;

use crate::{
    device::IOHandler,
    memory::{MemoryBus, MemoryRead, MemoryWrite},
};

const MAGIC_WORD: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_BANK_SIZE: usize = 0x4000;
const CHR_ROM_BANK_SIZE: usize = 0x2000;
const PRG_RAM_BANK_SIZE: usize = 0x2000;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub trainer: Vec<u8>,
    pub prg_ram: Vec<u8>,
    pub mapper: u8,
    pub mirroring: Mirroring,
    pub chr_ram: bool,
}

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

        Ok(Self {
            prg_rom,
            chr_rom,
            prg_ram,
            trainer,
            mapper,
            mirroring,
            chr_ram,
        })
    }
}

impl IOHandler for Rom {
    fn read(&mut self, mmu: &MemoryBus, address: u16) -> MemoryRead {
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

    fn write(&mut self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite {
        match address {
            0x6000..0x8000 => {
                if self.prg_ram.len() != 0 {
                    self.prg_ram[(address - 0x6000) as usize] = value;
                } else {
                    self.chr_rom[(address - 0x6000) as usize] = value;
                }
                MemoryWrite::Value(value)
            }
            _ => MemoryWrite::Block,
        }
    }
}
