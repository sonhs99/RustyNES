use super::util::*;
use super::{CPU_CLOCK, LENGTH_COUNTER_TABLE};

pub struct Pulse {
    duty: u8,
    halt: bool,
    envelope: Envelope,
    sweep: Sweep,

    timer: u16,
    length: u8,
}

impl Pulse {
    pub const fn new(ext: u8) -> Self {
        Self {
            duty: 0,
            halt: false,
            envelope: Envelope::new(),

            sweep: Sweep::new(ext),

            timer: 0,
            length: 0,
        }
    }

    pub fn update_1(&mut self, value: u8) {
        self.duty = value >> 6;
        self.halt = value & 0b0010_0000 != 0;
        self.envelope.set_disable(value & 0b0001_0000 != 0);
        self.envelope.set_volume(value & 0x0F);
    }

    pub fn update_2(&mut self, value: u8) {
        self.sweep.set_enable(value & 0b1000_0000 != 0);
        self.sweep.set_period((value >> 4) & 0b0111);
        self.sweep.set_negate(value & 0b0000_1000 != 0);
        self.sweep.set_shift(value & 0b0000_0111);
        self.sweep.reset();
    }

    pub fn update_3(&mut self, value: u8) {
        self.timer = self.timer & 0xFF00 | value as u16;
    }

    pub fn update_4(&mut self, value: u8) {
        self.length = LENGTH_COUNTER_TABLE[(value >> 3) as usize];
        self.timer = self.timer & 0x00FF | (value & 0b0000_0111) as u16;
        self.envelope.reset();
    }

    pub fn tick(&mut self, target_dev: u8) {
        if target_dev & 0x01 != 0 {
            self.envelope.tick(self.halt);
        }
        if target_dev & 0x02 != 0 {
            if self.length != 0 && !self.halt {
                self.length -= 1;
            }
            self.timer = self.sweep.tick(self.timer);
        }
        self.sweep.change(self.timer);
    }

    pub fn is_muted(&self) -> bool {
        self.sweep.mute() || self.length == 0
    }

    pub fn is_halt(&self) -> bool {
        self.halt
    }

    pub fn disable(&mut self) {
        self.length = 0;
    }

    pub fn value(&self) -> Tone {
        let duty = match self.duty {
            0 => WaveForm::Pulse12,
            1 => WaveForm::Pulse25,
            2 => WaveForm::Pulse50,
            3 => WaveForm::Pusle75,
            _ => unimplemented!(),
        };
        Tone {
            frequency: CPU_CLOCK / (16.0 * (self.timer as f64 + 1.0)),
            volume: if self.halt || self.is_muted() {
                0.0
            } else {
                (self.envelope.get_value() as f64 - 1.0) / 16.0
            },
            duty,
        }
    }
}
