use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::Duration;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SizedSample, Stream,
};
use fundsp::hacker::*;
use minifb::{Key, Scale, Window, WindowOptions};
use rustynes::{self, Frame, JoypadButton, Tone, WaveForm};

pub static SYSTEM_PALLETE: [(u8, u8, u8); 64] = [
    (0x80, 0x80, 0x80),
    (0x00, 0x3D, 0xA6),
    (0x00, 0x12, 0xB0),
    (0x44, 0x00, 0x96),
    (0xA1, 0x00, 0x5E),
    (0xC7, 0x00, 0x28),
    (0xBA, 0x06, 0x00),
    (0x8C, 0x17, 0x00),
    (0x5C, 0x2F, 0x00),
    (0x10, 0x45, 0x00),
    (0x05, 0x4A, 0x00),
    (0x00, 0x47, 0x2E),
    (0x00, 0x41, 0x66),
    (0x00, 0x00, 0x00),
    (0x05, 0x05, 0x05),
    (0x05, 0x05, 0x05),
    (0xC7, 0xC7, 0xC7),
    (0x00, 0x77, 0xFF),
    (0x21, 0x55, 0xFF),
    (0x82, 0x37, 0xFA),
    (0xEB, 0x2F, 0xB5),
    (0xFF, 0x29, 0x50),
    (0xFF, 0x22, 0x00),
    (0xD6, 0x32, 0x00),
    (0xC4, 0x62, 0x00),
    (0x35, 0x80, 0x00),
    (0x05, 0x8F, 0x00),
    (0x00, 0x8A, 0x55),
    (0x00, 0x99, 0xCC),
    (0x21, 0x21, 0x21),
    (0x09, 0x09, 0x09),
    (0x09, 0x09, 0x09),
    (0xFF, 0xFF, 0xFF),
    (0x0F, 0xD7, 0xFF),
    (0x69, 0xA2, 0xFF),
    (0xD4, 0x80, 0xFF),
    (0xFF, 0x45, 0xF3),
    (0xFF, 0x61, 0x8B),
    (0xFF, 0x88, 0x33),
    (0xFF, 0x9C, 0x12),
    (0xFA, 0xBC, 0x20),
    (0x9F, 0xE3, 0x0E),
    (0x2B, 0xF0, 0x35),
    (0x0C, 0xF0, 0xA4),
    (0x05, 0xFB, 0xFF),
    (0x5E, 0x5E, 0x5E),
    (0x0D, 0x0D, 0x0D),
    (0x0D, 0x0D, 0x0D),
    (0xFF, 0xFF, 0xFF),
    (0xA6, 0xFC, 0xFF),
    (0xB3, 0xEC, 0xFF),
    (0xDA, 0xAB, 0xEB),
    (0xFF, 0xA8, 0xF9),
    (0xFF, 0xAB, 0xB3),
    (0xFF, 0xD2, 0xB0),
    (0xFF, 0xEF, 0xA6),
    (0xFF, 0xF7, 0x9C),
    (0xD7, 0xE8, 0x95),
    (0xA6, 0xED, 0xAF),
    (0xA2, 0xF2, 0xDA),
    (0x99, 0xFF, 0xFC),
    (0xDD, 0xDD, 0xDD),
    (0x11, 0x11, 0x11),
    (0x11, 0x11, 0x11),
];

pub struct Hardware {
    window: Window,
    ch1_fr: Shared<f64>,
    ch1_vo: Shared<f64>,
    ch1_duty: Shared<f64>,
    ch2_fr: Shared<f64>,
    ch2_vo: Shared<f64>,
    ch2_duty: Shared<f64>,
    ch3_fr: Shared<f64>,
    ch3_vo: Shared<f64>,
    key_mapper: [(Key, JoypadButton); 8],
}

impl Hardware {
    pub fn new() -> Self {
        let window = Window::new(
            "rustynes",
            rustynes::WIDTH,
            rustynes::HEIGHT,
            WindowOptions {
                resize: false,
                scale: Scale::X2,
                ..WindowOptions::default()
            },
        )
        .unwrap_or_else(|e| {
            panic!("{e}");
        });

        let ch1_fr = Shared::new(0.0);
        let ch1_vo = Shared::new(0.0);
        let ch1_duty = Shared::new(0.0);
        let ch2_fr = Shared::new(0.0);
        let ch2_vo = Shared::new(0.0);
        let ch2_duty = Shared::new(0.0);
        let ch3_fr = Shared::new(0.0);
        let ch3_vo = Shared::new(0.0);

        let sound = Arc::new(Mutex::new(0.0f64));
        run_audio(
            ch1_fr.clone(),
            ch1_vo.clone(),
            ch1_duty.clone(),
            ch2_fr.clone(),
            ch2_vo.clone(),
            ch2_duty.clone(),
            ch3_fr.clone(),
            ch3_vo.clone(),
        );

        Self {
            window,
            ch1_fr,
            ch1_vo,
            ch1_duty,
            ch2_fr,
            ch2_vo,
            ch2_duty,
            ch3_fr,
            ch3_vo,
            key_mapper: [
                (Key::Down, JoypadButton::Down),
                (Key::Up, JoypadButton::Up),
                (Key::Left, JoypadButton::Left),
                (Key::Right, JoypadButton::Right),
                (Key::Space, JoypadButton::Select),
                (Key::Enter, JoypadButton::Start),
                (Key::Z, JoypadButton::ButtonB),
                (Key::X, JoypadButton::ButtonA),
            ],
        }
    }
}

impl rustynes::Hardware for Hardware {
    fn is_active(&mut self) -> bool {
        self.window.is_open()
    }

    fn draw_framebuffer(&mut self, frame_buffer: &Frame) {
        let mut frame = [0u32; rustynes::WIDTH * rustynes::HEIGHT];
        for idx in 0..rustynes::WIDTH * rustynes::HEIGHT {
            let color = SYSTEM_PALLETE[frame_buffer.data[idx] as usize];
            frame[idx] =
                0xFF000000 | (color.0 as u32) << 16 | (color.1 as u32) << 8 | color.2 as u32;
        }
        self.window
            .update_with_buffer(&frame, rustynes::WIDTH, rustynes::HEIGHT)
            .unwrap();
    }

    fn pad_p1(&mut self) -> rustynes::JoypadButton {
        let mut current_state = JoypadButton::from_bits_truncate(0x00);
        for (key, joypad) in self.key_mapper {
            current_state.set(joypad, self.window.is_key_down(key));
        }
        current_state
    }

    fn pad_p2(&mut self) -> rustynes::JoypadButton {
        JoypadButton::from_bits_truncate(0x00)
    }

    fn play_sound(&mut self, sound: [Tone; 3]) {
        self.ch1_fr.set_value(sound[0].frequency);
        self.ch1_vo.set_value(sound[0].volume);
        self.ch1_duty.set_value(match sound[0].duty {
            WaveForm::Pulse12 => 0.125,
            WaveForm::Pulse25 => 0.25,
            WaveForm::Pulse50 => 0.50,
            WaveForm::Pusle75 => 0.75,
            WaveForm::Triangle => todo!(),
        } as f64);

        self.ch2_fr.set_value(sound[1].frequency);
        self.ch2_vo.set_value(sound[1].volume);
        self.ch2_duty.set_value(match sound[1].duty {
            WaveForm::Pulse12 => 0.125,
            WaveForm::Pulse25 => 0.25,
            WaveForm::Pulse50 => 0.50,
            WaveForm::Pusle75 => 0.75,
            WaveForm::Triangle => todo!(),
        } as f64);

        // self.ch3_fr.set_value(sound[2].frequency);
        // self.ch3_vo.set_value(1.0);
    }
}

fn run_audio(
    ch1_fr: Shared<f64>,
    ch1_vo: Shared<f64>,
    ch1_duty: Shared<f64>,
    ch2_fr: Shared<f64>,
    ch2_vo: Shared<f64>,
    ch2_duty: Shared<f64>,
    ch3_fr: Shared<f64>,
    ch3_vo: Shared<f64>,
) {
    spawn(move || {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap().config();
        let channel = config.channels as usize;

        let ch1_mono = ((var(&ch1_fr) | var(&ch1_duty)) >> pulse()) * var(&ch1_vo) * constant(0.33);
        let ch2_mono = ((var(&ch2_fr) | var(&ch2_duty)) >> pulse()) * var(&ch2_vo) * constant(0.33);
        let ch3_mono = (var(&ch3_fr) >> triangle()) * var(&ch3_vo) * constant(0.33);

        let mut total_mono = ch1_mono + ch2_mono + ch3_mono;

        let mut next_sample = move || total_mono.get_mono();

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channel, &mut next_sample)
                },
                |err| {
                    eprintln!("an error occurred on stream: {}", err);
                },
                None,
            )
            .unwrap();

        stream.play().unwrap();

        loop {
            std::thread::sleep(Duration::from_millis(1));
        }
    });
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f64)
where
    T: SizedSample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        for sample in frame.iter_mut() {
            let volume = next_sample();
            *sample = T::from_sample(volume as f32);
        }
    }
}
