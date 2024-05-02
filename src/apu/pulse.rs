use libc_print::libc_println;

use super::CPU_CLOCK;
use super::{util::*, TimeEvent};

pub struct Pulse {
    duty: usize,
    halt: bool,
    envelope: Envelope,
    sweep: Sweep,

    sequence: Sequence,
    length: LengthCounter,
}

impl Pulse {
    pub fn new(ext: u8) -> Self {
        Self {
            duty: 0,
            halt: false,
            envelope: Envelope::new(),

            sweep: Sweep::new(ext),

            sequence: Sequence::new(8),
            length: LengthCounter::new(),
        }
    }

    pub fn update_1(&mut self, value: u8) {
        self.duty = (value >> 6) as usize;
        self.halt = value & 0b0010_0000 != 0;
        self.length.set_enable(!self.halt);
        self.envelope.set_disable(value & 0b0001_0000 != 0);
        self.envelope.set_volume(value & 0x0F);
        self.sequence.reset();
    }

    pub fn update_2(&mut self, value: u8) {
        self.sweep.set_enable(value & 0b1000_0000 != 0);
        self.sweep.set_period((value >> 4) & 0b0111);
        self.sweep.set_negate(value & 0b0000_1000 != 0);
        self.sweep.set_shift(value & 0b0000_0111);
        self.sweep.reset();
    }

    pub fn update_3(&mut self, value: u8) {
        self.sequence.set_timer_low(value);
    }

    pub fn update_4(&mut self, value: u8) {
        self.length.set_length(value >> 3);
        self.sequence.set_timer_high((value & 0b0000_0111));
        self.envelope.reset();
    }

    pub fn tick(&mut self, time_event: &TimeEvent) {
        if time_event.contains(TimeEvent::APUClock) {
            self.sequence.tick();
            self.sweep.change(self.sequence.period());
        }
        if time_event.contains(TimeEvent::QuarterFrame) {
            self.envelope.tick(self.halt);
        }
        if time_event.contains(TimeEvent::HalfFrame) {
            self.length.tick();
            let new_period = self.sweep.tick(self.sequence.period());
            if new_period != self.sequence.period() {
                self.sequence.set_timer_high((new_period >> 8) as u8);
                self.sequence.set_timer_low(new_period as u8);
            }
        }
    }

    pub fn is_halt(&self) -> bool {
        self.halt
    }

    pub fn is_end(&self) -> bool {
        self.length.is_end()
    }

    pub fn disable(&mut self) {
        self.length.disable();
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
            frequency: CPU_CLOCK / (16.0 * (self.sequence.period() as f64 + 1.0)),
            volume: if self.halt
                || self.sweep.is_mute()
                || self.length.is_mute()
                || self.sequence.is_mute()
            {
                0.0
            } else {
                (self.envelope.value() as f64) / 15.0
            },
            duty,
        }
    }
}
