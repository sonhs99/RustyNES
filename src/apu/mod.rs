use libc_print::libc_println;

use crate::{
    device::IOHandler,
    memory::{MemoryBus, MemoryRead, MemoryWrite},
};

pub use self::util::{Tone, WaveForm};
use self::{pulse::Pulse, triangle::Triangle};

mod noise;
mod pulse;
mod triangle;
mod util;

const PULSE_SEQUENCE_TABLE: [u8; 4] = [0b0000_0001, 0b0000_0011, 0b0000_1111, 0b1111_1100];
const LENGTH_COUNTER_TABLE: [u8; 0x20] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];
const CPU_CLOCK: f64 = 1789773.0 / 2.0;

pub struct FrameCounter {
    index: usize,
    mode5: bool,
    interrupt: bool,
}

impl FrameCounter {
    pub fn new() -> Self {
        Self {
            index: 0,
            mode5: false,
            interrupt: false,
        }
    }

    pub fn next(&mut self) -> u8 {
        const MODE5_TABLE: [u8; 5] = [1, 3, 1, 0, 3];
        const MODE4_TABLE: [u8; 4] = [1, 3, 1, 7];
        if self.mode5 {
            let clock = MODE5_TABLE[self.index];
            self.index += 1;
            self.index %= 5;
            clock
        } else {
            let clock = MODE4_TABLE[self.index];
            self.index += 1;
            self.index %= 4;
            clock
        }
    }

    pub fn set(&mut self, mode5: bool, interrupt: bool) {
        self.mode5 = mode5;
        self.interrupt = interrupt;
        self.index = if mode5 { 4 } else { 3 };
    }
}

pub struct Apu {
    pulse_1: Pulse,
    pulse_2: Pulse,
    triangle: Triangle,
    ctrl: u8,
    status: u8,
    frame_counter: FrameCounter,
    cpu_cycles: usize,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            pulse_1: Pulse::new(1),
            pulse_2: Pulse::new(0),
            triangle: Triangle::new(),
            ctrl: 0,
            status: 0,
            frame_counter: FrameCounter::new(),
            cpu_cycles: 0,
        }
    }

    pub fn step(&mut self, cpu_cycle: u16) -> [Tone; 3] {
        let apu_cycles = (cpu_cycle as usize + self.cpu_cycles & 0x01) / 2;
        self.cpu_cycles += cpu_cycle as usize;

        for _ in 0..apu_cycles {
            let target_dev = self.frame_counter.next();
            if !self.pulse_1.is_halt() {
                self.pulse_1.tick(target_dev);
            }
            if !self.pulse_2.is_halt() {
                self.pulse_2.tick(target_dev);
            }
            if !self.triangle.is_halt() {
                self.triangle.tick(target_dev);
            }

            if target_dev & 0x04 != 0 && self.frame_counter.interrupt {
                self.status |= 0x40;
            }
        }

        if self.pulse_1.is_muted() {
            self.status |= 0b0000_0001;
        } else {
            self.status &= 0b1111_1110;
        }
        if self.pulse_2.is_muted() {
            self.status |= 0b0000_0010;
        } else {
            self.status &= 0b1111_1101;
        }
        if self.triangle.is_muted() {
            self.status |= 0b0000_0100;
        } else {
            self.status &= 0b1111_1011;
        }

        let p1_volume = self.pulse_1.value();
        let p2_volume = self.pulse_2.value();
        let tri_volume = self.triangle.value();
        [p1_volume, p2_volume, tri_volume]
    }
}

impl IOHandler for Apu {
    fn read(&mut self, mmu: &MemoryBus, address: u16) -> MemoryRead {
        match address {
            0x4015 => {
                let value = self.status;
                self.status &= 0b1011_1111;
                MemoryRead::Value(value)
            }
            0x4017 => MemoryRead::Pass,
            _ => panic!("[APU] Attemp to Read Write-Only register: ${address:04X}"),
        }
    }

    fn write(&mut self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite {
        // libc_println!("[APU] {address:04X} = {value:02X}");
        match address {
            0x4000 => {
                self.pulse_1.update_1(value);
                // libc_println!("[APU] {:?}", self.pulse_1.envelope);
                MemoryWrite::Value(value)
            }
            0x4001 => {
                self.pulse_1.update_2(value);
                MemoryWrite::Value(value)
            }
            0x4002 => {
                self.pulse_1.update_3(value);
                MemoryWrite::Value(value)
            }
            0x4003 => {
                self.pulse_1.update_4(value);
                MemoryWrite::Value(value)
            }
            0x4004 => {
                self.pulse_2.update_1(value);
                MemoryWrite::Value(value)
            }
            0x4005 => {
                self.pulse_2.update_2(value);
                MemoryWrite::Value(value)
            }
            0x4006 => {
                self.pulse_2.update_3(value);
                MemoryWrite::Value(value)
            }
            0x4007 => {
                self.pulse_2.update_4(value);
                MemoryWrite::Value(value)
            }
            0x4008 => {
                self.triangle.update_1(value);
                MemoryWrite::Value(value)
            }
            0x4009 => {
                self.triangle.update_2(value);
                MemoryWrite::Value(value)
            }
            0x400A => {
                self.triangle.update_3(value);
                MemoryWrite::Value(value)
            }
            0x400B => {
                self.triangle.update_4(value);
                MemoryWrite::Value(value)
            }
            0x400C => MemoryWrite::Pass,
            0x400D => MemoryWrite::Pass,
            0x400E => MemoryWrite::Pass,
            0x400F => MemoryWrite::Pass,
            0x4010 => MemoryWrite::Pass,
            0x4011 => MemoryWrite::Pass,
            0x4012 => MemoryWrite::Pass,
            0x4013 => MemoryWrite::Pass,
            0x4015 => {
                self.ctrl = value;
                if value & 0x01 == 0 {
                    self.pulse_1.disable();
                }
                if value & 0x02 == 0 {
                    self.pulse_2.disable();
                }
                if value & 0x04 == 0 {
                    self.pulse_2.disable();
                }
                MemoryWrite::Value(value)
            }
            0x4017 => {
                self.frame_counter
                    .set(value & 0b1000_0000 != 0, value & 0b0100_0000 != 0);
                MemoryWrite::Value(value)
            }
            _ => MemoryWrite::Block,
        }
    }
}
