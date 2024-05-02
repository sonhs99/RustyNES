use libc_print::libc_println;

use super::util::*;
use super::TimeEvent;
use super::CPU_CLOCK;

const NOISE_PERIOD_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

struct FeedbackRegister {
    register: u16,
    mode: bool,
    period: u16,
    counter: u16,
}

impl FeedbackRegister {
    pub fn new() -> Self {
        Self {
            register: 1,
            mode: false,
            period: 0,
            counter: 0,
        }
    }

    pub fn set_mode(&mut self, mode: bool) {
        self.mode = mode;
    }

    pub fn set_timer(&mut self, period: u16) {
        self.period = period;
        self.counter = period;
    }

    pub fn tick(&mut self) {
        if self.counter == 0 {
            self.counter = self.period;
            let feedback = (self.register & 0x01)
                ^ if self.mode {
                    (self.register >> 5) & 0x01
                } else {
                    (self.register >> 1) & 0x01
                };
            self.register = (self.register >> 1) | feedback << 14;
        } else {
            self.counter -= self.period;
        }
    }

    pub fn is_mute(&self) -> bool {
        self.register & 0x01 == 0
    }
}

pub struct Noise {
    halt: bool,
    envelope: Envelope,

    feedback: FeedbackRegister,
    length: LengthCounter,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            halt: false,
            envelope: Envelope::new(),

            feedback: FeedbackRegister::new(),
            length: LengthCounter::new(),
        }
    }

    pub fn update_1(&mut self, value: u8) {
        self.halt = value & 0b0010_0000 != 0;
        self.length.set_enable(!self.halt);
        self.envelope.set_disable(value & 0b0001_0000 != 0);
        self.envelope.set_volume(value & 0x0F);
    }

    pub fn update_2(&mut self, _value: u8) {}

    pub fn update_3(&mut self, value: u8) {
        self.feedback.set_mode(value & 0x80 != 0);
        self.feedback
            .set_timer(NOISE_PERIOD_TABLE[(value & 0x0F) as usize])
    }

    pub fn update_4(&mut self, value: u8) {
        self.length.set_length(value >> 3);
        self.envelope.reset();
    }

    pub fn tick(&mut self, time_event: &TimeEvent) {
        if time_event.contains(TimeEvent::APUClock) {
            self.feedback.tick();
        }
        if time_event.contains(TimeEvent::QuarterFrame) {
            self.envelope.tick(self.halt);
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
            frequency: 0.0,
            volume: if self.halt || self.length.is_mute() || self.feedback.is_mute() {
                0.0
            } else {
                (self.envelope.value() as f64) / 15.0
            },
            duty: WaveForm::Noise,
        }
    }
}
