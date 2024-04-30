use super::util::*;
use super::{CPU_CLOCK, LENGTH_COUNTER_TABLE};

pub struct Triangle {
    halt: bool,
    linear: u8,

    timer: u16,
    length: u8,
}

impl Triangle {
    pub const fn new() -> Self {
        Self {
            halt: false,
            linear: 0,

            timer: 0,
            length: 0,
        }
    }

    pub fn update_1(&mut self, value: u8) {
        self.halt = value & 0b1000_0000 != 0;
        self.linear = value & 0b0111_1111;
    }

    pub fn update_2(&mut self, value: u8) {}

    pub fn update_3(&mut self, value: u8) {
        self.timer = self.timer & 0xFF00 | value as u16;
    }

    pub fn update_4(&mut self, value: u8) {
        self.length = LENGTH_COUNTER_TABLE[(value >> 3) as usize];
        self.timer = self.timer & 0x00FF | (value & 0b0000_0111) as u16;
    }

    pub fn tick(&mut self, target_dev: u8) {
        if target_dev & 0x02 != 0 {
            if self.length != 0 && !self.halt {
                self.length -= 1;
            }
        }
    }

    pub fn is_muted(&self) -> bool {
        self.length == 0
    }

    pub fn is_halt(&self) -> bool {
        self.halt
    }

    pub fn disable(&mut self) {
        self.length = 0;
    }

    pub fn value(&self) -> Tone {
        Tone {
            frequency: CPU_CLOCK / (32.0 * (self.timer as f64 + 1.0)),
            volume: if self.halt || self.is_muted() {
                0.0
            } else {
                ((self.linear >> 3) as f64 - 1.0) / 16.0
            },
            duty: WaveForm::Triangle,
        }
    }
}
