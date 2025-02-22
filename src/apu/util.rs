pub const LENGTH_COUNTER_TABLE: [u8; 0x20] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

#[derive(Debug)]
pub enum WaveForm {
    Pulse12,
    Pulse25,
    Pulse50,
    Pusle75,
    Triangle,
    Noise,
}

#[derive(Debug)]
pub struct Tone {
    pub frequency: f64,
    pub volume: f64,
    pub duty: WaveForm,
}

pub struct Envelope {
    constant: bool,
    volume: u8,

    start_flag: bool,
    decay_level: u8,
    current_value: u8,
}

impl Envelope {
    pub const fn new() -> Self {
        Self {
            start_flag: false,
            constant: false,
            volume: 0,
            decay_level: 0,
            current_value: 0,
        }
    }

    pub fn set_volume(&mut self, period: u8) {
        self.volume = period;
        self.current_value = period;
    }

    pub fn set_disable(&mut self, constant: bool) {
        self.constant = constant
    }

    pub fn tick(&mut self, halt: bool) {
        if !self.start_flag {
            if self.current_value != 0 {
                self.current_value -= 1;
            } else {
                self.current_value = self.volume;
                if self.decay_level == 0 {
                    if !halt {
                        self.decay_level = 15;
                    }
                } else {
                    self.decay_level -= 1;
                }
            }
        } else {
            self.start_flag = false;
            self.decay_level = 15;
            self.current_value = self.volume;
        }
    }

    pub fn value(&self) -> u8 {
        if self.constant {
            self.volume
        } else {
            self.decay_level
        }
    }

    pub fn reset(&mut self) {
        self.start_flag = true;
    }
}

pub struct Sweep {
    enable: bool,
    period: u8,
    negate: bool,
    shift: u8,

    current_value: u8,
    reload_flag: bool,
    mute: bool,
    ext: u8,
    change: u16,
}

impl Sweep {
    pub const fn new(ext: u8) -> Self {
        Self {
            enable: false,
            period: 0,
            negate: false,
            shift: 0,

            current_value: 0,
            reload_flag: false,
            mute: false,
            change: 0,
            ext,
        }
    }

    pub fn set_enable(&mut self, enable: bool) {
        self.enable = enable;
    }

    pub fn set_period(&mut self, period: u8) {
        self.period = period + 1;
    }

    pub fn set_negate(&mut self, negate: bool) {
        self.negate = negate;
    }

    pub fn set_shift(&mut self, shift: u8) {
        self.shift = shift;
    }

    pub fn change(&mut self, timer: u16) {
        if self.enable {
            self.change = timer >> self.shift;
            self.mute = timer < 8 || timer + self.change > 0x07FF;
        }
    }

    pub fn is_mute(&self) -> bool {
        self.mute
    }

    pub fn tick(&mut self, timer: u16) -> u16 {
        let next_period = if self.enable && self.current_value != 0 && self.shift != 0 && !self.mute
        {
            if timer >= 8 && self.change <= 0x07FF {
                match self.negate {
                    true => timer
                        .checked_sub(self.change + self.ext as u16)
                        .unwrap_or(0),
                    false => timer.checked_add(self.change).unwrap_or(u16::MAX),
                }
            } else {
                timer
            }
        } else {
            timer
        };
        if self.current_value == 0 || self.reload_flag {
            self.current_value = self.period;
            self.reload_flag = false;
        } else {
            self.current_value -= 1;
        }
        self.mute = timer < 8 || timer > 0x7FF;
        next_period
    }

    pub fn reset(&mut self) {
        self.reload_flag = true;
    }
}

pub struct LengthCounter {
    enable: bool,
    length: u8,
}

impl LengthCounter {
    pub fn new() -> Self {
        Self {
            enable: false,
            length: 0,
        }
    }

    pub fn set_enable(&mut self, flag: bool) {
        self.enable = flag;
    }

    pub fn set_length(&mut self, length: u8) {
        self.length = LENGTH_COUNTER_TABLE[length as usize];
    }

    pub fn disable(&mut self) {
        self.enable = false;
        self.length = 0;
    }

    pub fn is_mute(&self) -> bool {
        !self.enable || self.length == 0
    }

    pub fn is_end(&self) -> bool {
        self.length == 0
    }

    pub fn tick(&mut self) {
        if self.length != 0 {
            self.length -= 1;
        }
    }
}

pub struct Sequence {
    sequence: u8,
    sequence_counter: u8,
    timer: u16,
    timer_counter: u16,
}

impl Sequence {
    pub fn new(sequence: u8) -> Self {
        Self {
            sequence,
            sequence_counter: 0,
            timer: 0,
            timer_counter: 0,
        }
    }

    pub fn set_timer_low(&mut self, value: u8) {
        self.timer = self.timer & 0xFF00 | value as u16;
    }

    pub fn set_timer_high(&mut self, value: u8) {
        self.timer = self.timer & 0x00FF | (value as u16) << 8;
        self.timer_counter = self.timer;
    }

    pub fn tick(&mut self) {
        if self.timer_counter == 0 {
            self.timer_counter = self.timer;
            self.sequence_counter = (self.sequence_counter + 1) % self.sequence;
        } else {
            self.timer_counter -= 1;
        }
    }

    pub fn value(&self) -> u8 {
        self.sequence_counter
    }

    pub fn period(&self) -> u16 {
        self.timer
    }

    pub fn reset(&mut self) {
        self.sequence_counter = 0;
    }

    pub fn is_mute(&self) -> bool {
        self.timer < 8
    }
}
