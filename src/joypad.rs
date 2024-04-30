use bitflags::bitflags;
use libc_print::libc_println;

use crate::{
    device::IOHandler,
    memory::{MemoryBus, MemoryRead, MemoryWrite},
};

bitflags! {
    #[derive(Clone, Copy)]
    pub struct JoypadButton: u8 {
        const Right             = 0b10000000;
        const Left              = 0b01000000;
        const Down              = 0b00100000;
        const Up                = 0b00010000;
        const Start             = 0b00001000;
        const Select            = 0b00000100;
        const ButtonB           = 0b00000010;
        const ButtonA           = 0b00000001;
    }
}

#[derive(Clone, Copy)]
pub struct Joypad {
    pub status: [JoypadButton; 2],
    index: [u8; 2],
    ctrl: u8,
}

impl Joypad {
    pub const fn new() -> Self {
        Self {
            status: [JoypadButton::from_bits_truncate(0x00); 2],
            index: [0; 2],
            ctrl: 0,
        }
    }

    pub fn update(&mut self, button_p1: JoypadButton, button_p2: JoypadButton) {
        self.status[0] = button_p1;
        self.status[1] = button_p2;
        // libc_println!("[Joypad] $4016 = {:02X} [U]", self.status.bits());
    }

    pub fn get(&mut self, idx: usize) -> u8 {
        if self.index[idx] > 7 {
            1
        } else {
            let value = (self.status[idx].bits() >> self.index[idx]) & 0x01;
            // libc_println!(
            //     "[Joypad] ${:04X} = {:02X}, S = {:02X} I = {} [R]",
            //     address,
            //     value,
            //     self.status.bits(),
            //     self.index
            // );
            if self.ctrl & 0x01 == 0 {
                self.index[idx] += 1;
            }
            value
        }
    }
}

impl IOHandler for Joypad {
    fn read(&mut self, _mmu: &MemoryBus, address: u16) -> MemoryRead {
        match address {
            0x4016 | 0x4017 => MemoryRead::Value(self.get(address as usize - 0x4016)),
            _ => MemoryRead::Pass,
        }
    }
    fn write(&mut self, _mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite {
        match address {
            0x4016 => {
                self.ctrl = value & 0b0000_0111;
                if value & 0x01 != 0 {
                    self.index[0] = 0;
                    self.index[1] = 0;
                }
                MemoryWrite::Value(value)
            }
            _ => MemoryWrite::Pass,
        }
    }
}
