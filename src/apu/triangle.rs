use libc_print::libc_println;

use super::util::*;
use super::TimeEvent;
use super::CPU_CLOCK;
use super::PITCH_RATIO;

pub struct LinearCounter {
    enable: bool,
    reload: bool,
    reload_value: u8,
    counter: u8,
}

impl LinearCounter {
    pub fn new() -> Self {
        Self {
            enable: false,
            reload: false,
            reload_value: 0,
            counter: 0,
        }
    }

    pub fn set_enable(&mut self, flag: bool) {
        self.enable = flag;
    }

    pub fn set_counter(&mut self, counter: u8) {
        self.reload_value = counter;
    }

    pub fn is_mute(&self) -> bool {
        !self.enable || self.counter != 0
    }

    pub fn reset(&mut self) {
        self.reload = true;
    }

    pub fn tick(&mut self) {
        if self.reload {
            self.reload = false;
            self.counter = self.reload_value;
        } else if self.counter != 0 {
            self.counter -= 1;
        }
    }
}

pub struct Triangle {
    halt: bool,
    linear: LinearCounter,

    sequence: Sequence,
    length: LengthCounter,
}

impl Triangle {
    pub fn new() -> Self {
        Self {
            halt: false,
            linear: LinearCounter::new(),

            sequence: Sequence::new(32),
            length: LengthCounter::new(),
        }
    }

    pub fn update_1(&mut self, value: u8) {
        self.halt = value & 0b1000_0000 != 0;
        self.linear.set_enable(!self.halt);
        self.length.set_enable(!self.halt);
        self.linear.set_counter(value & 0b0111_1111);
        self.sequence.reset();
    }

    pub fn update_2(&mut self, _value: u8) {}

    pub fn update_3(&mut self, value: u8) {
        self.sequence.set_timer_low(value);
    }

    pub fn update_4(&mut self, value: u8) {
        self.length.set_length(value >> 3);
        self.sequence.set_timer_high(value & 0b0000_0111);
        self.linear.reset();
    }

    pub fn tick(&mut self, time_event: &TimeEvent) {
        if time_event.contains(TimeEvent::APUClock) {
            if !self.length.is_mute() && !self.linear.is_mute() {
                self.sequence.tick();
            }
        }
        if time_event.contains(TimeEvent::QuarterFrame) {
            self.linear.tick();
        }
        if time_event.contains(TimeEvent::HalfFrame) {
            self.length.tick();
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
        Tone {
            frequency: CPU_CLOCK
                / (32.0 * (self.sequence.period() as f64 + 1.0))
                / PITCH_RATIO as f64,
            volume: if self.halt || self.length.is_mute() || self.sequence.is_mute() {
                0.0
            } else {
                1.0
            },
            duty: WaveForm::Triangle,
        }
    }
}
