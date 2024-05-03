use crate::{
    device::IOHandler,
    memory::{MemoryBus, MemoryRead, MemoryWrite},
};
use bitflags::bitflags;
use libc_print::libc_println;

pub use self::util::{Tone, WaveForm};
use self::{noise::Noise, pulse::Pulse, triangle::Triangle};

mod noise;
mod pulse;
mod triangle;
mod util;

const CLOCK_RATIO: usize = 1;
const PITCH_RATIO: usize = 3;
const CPU_CLOCK: f64 = 1789773.0 / CLOCK_RATIO as f64;

bitflags! {
    pub struct TimeEvent: u8 {
        const CPUClock = 0b0000_0001;
        const APUClock = 0b0000_0010;
        const QuarterFrame = 0b0000_0100;
        const HalfFrame = 0b0000_1000;
        const Interrupt = 0b0001_0000;
    }
}

pub struct FrameCounter {
    timer: usize,
    mode5: bool,
    interrupt: bool,
}

impl FrameCounter {
    pub fn new() -> Self {
        Self {
            timer: 0,
            mode5: false,
            interrupt: false,
        }
    }

    pub fn next(&mut self) -> TimeEvent {
        let mut event = TimeEvent::from_bits_truncate(0x00);
        self.timer = (self.timer + 1)
            % (if self.mode5 {
                37281 * CLOCK_RATIO
            } else {
                29830 * CLOCK_RATIO
            });

        let time_table = if self.mode5 {
            [
                7457 * CLOCK_RATIO,
                14913 * CLOCK_RATIO,
                22371 * CLOCK_RATIO,
                37281 * CLOCK_RATIO,
            ]
        } else {
            [
                7457 * CLOCK_RATIO,
                14913 * CLOCK_RATIO,
                22371 * CLOCK_RATIO,
                29829 * CLOCK_RATIO,
            ]
        };

        event.set(TimeEvent::CPUClock, self.timer % CLOCK_RATIO == 0);
        event.set(TimeEvent::APUClock, self.timer % CLOCK_RATIO * 2 == 0);
        event.set(TimeEvent::QuarterFrame, time_table.contains(&self.timer));
        event.set(
            TimeEvent::HalfFrame,
            self.timer == time_table[1] || self.timer == time_table[3],
        );
        if !self.mode5 && !self.interrupt {
            event.set(
                TimeEvent::Interrupt,
                self.timer >= time_table[3] - 1 && self.timer <= time_table[3] + 3,
            );
        }

        event
    }

    pub fn set(&mut self, mode5: bool, interrupt: bool) {
        self.mode5 = mode5;
        self.interrupt = interrupt;
        self.timer = 0;
    }
}

pub struct Apu {
    pulse_1: Pulse,
    pulse_2: Pulse,
    triangle: Triangle,
    noise: Noise,
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
            noise: Noise::new(),
            ctrl: 0,
            status: 0,
            frame_counter: FrameCounter::new(),
            cpu_cycles: 0,
        }
    }

    pub fn step(&mut self, cpu_cycle: u16) -> [Tone; 4] {
        // let apu_cycles = (cpu_cycle as usize + self.cpu_cycles & 0x01) / 2;/
        self.cpu_cycles += cpu_cycle as usize;

        for _ in 0..cpu_cycle {
            let event = self.frame_counter.next();
            if !self.pulse_1.is_halt() {
                self.pulse_1.tick(&event);
            }
            if !self.pulse_2.is_halt() {
                self.pulse_2.tick(&event);
            }
            if !self.triangle.is_halt() {
                self.triangle.tick(&event);
            }
            if !self.noise.is_halt() {
                self.noise.tick(&event);
            }

            if event.contains(TimeEvent::Interrupt) {
                self.status |= 0x40;
            }
        }

        if self.pulse_1.is_end() {
            self.status |= 0b0000_0001;
        } else {
            self.status &= 0b1111_1110;
        }
        if self.pulse_2.is_end() {
            self.status |= 0b0000_0010;
        } else {
            self.status &= 0b1111_1101;
        }
        if self.triangle.is_end() {
            self.status |= 0b0000_0100;
        } else {
            self.status &= 0b1111_1011;
        }
        if self.noise.is_end() {
            self.status |= 0b0000_1000;
        } else {
            self.status &= 0b1111_0111;
        }

        let p1_volume = self.pulse_1.value();
        let p2_volume = self.pulse_2.value();
        let tri_volume = self.triangle.value();
        let noise_volume = self.noise.value();
        [p1_volume, p2_volume, tri_volume, noise_volume]
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
        match address {
            0x4000 => {
                self.pulse_1.update_1(value);
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
            0x400C => {
                self.noise.update_1(value);
                MemoryWrite::Value(value)
            }
            0x400D => {
                self.noise.update_2(value);
                MemoryWrite::Value(value)
            }
            0x400E => {
                self.noise.update_3(value);
                MemoryWrite::Value(value)
            }
            0x400F => {
                self.noise.update_4(value);
                MemoryWrite::Value(value)
            }
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
                    self.triangle.disable();
                }
                if value & 0x08 == 0 {
                    self.noise.disable();
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
