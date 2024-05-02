use std::thread::spawn;
use std::time::Duration;

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, SizedSample, Stream,
};
use fundsp::hacker::*;
use minifb::{Key, Scale, Window, WindowOptions};
use rustynes::{self, Frame, JoypadButton, Tone, WaveForm};

pub static SYSTEM_PALLETE: [u32; 64] = [
    0xFF808080, 0xFF003DA6, 0xFF0012B0, 0xFF440096, 0xFFA1005E, 0xFFC70028, 0xFFBA0600, 0xFF8C1700,
    0xFF5C2F00, 0xFF104500, 0xFF054A00, 0xFF00472E, 0xFF004166, 0xFF000000, 0xFF050505, 0xFF050505,
    0xFFC7C7C7, 0xFF0077FF, 0xFF2155FF, 0xFF8237FA, 0xFFEB2FB5, 0xFFFF2950, 0xFFFF2200, 0xFFD63200,
    0xFFC46200, 0xFF358000, 0xFF058F00, 0xFF008A55, 0xFF0099CC, 0xFF212121, 0xFF090909, 0xFF090909,
    0xFFFFFFFF, 0xFF0FD7FF, 0xFF69A2FF, 0xFFD480FF, 0xFFFF45F3, 0xFFFF618B, 0xFFFF8833, 0xFFFF9C12,
    0xFFFABC20, 0xFF9FE30E, 0xFF2BF035, 0xFF0CF0A4, 0xFF05FBFF, 0xFF5E5E5E, 0xFF0D0D0D, 0xFF0D0D0D,
    0xFFFFFFFF, 0xFFA6FCFF, 0xFFB3ECFF, 0xFFDAABEB, 0xFFFFA8F9, 0xFFFFABB3, 0xFFFFD2B0, 0xFFFFEFA6,
    0xFFFFF79C, 0xFFD7E895, 0xFFA6EDAF, 0xFFA2F2DA, 0xFF99FFFC, 0xFFDDDDDD, 0xFF111111, 0xFF111111,
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
    ch4_fr: Shared<f64>,
    ch4_vo: Shared<f64>,
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
        let ch4_fr = Shared::new(0.0);
        let ch4_vo = Shared::new(0.0);

        run_audio(
            ch1_fr.clone(),
            ch1_vo.clone(),
            ch1_duty.clone(),
            ch2_fr.clone(),
            ch2_vo.clone(),
            ch2_duty.clone(),
            ch3_fr.clone(),
            ch3_vo.clone(),
            ch4_fr.clone(),
            ch4_vo.clone(),
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
            ch4_fr,
            ch4_vo,
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
            frame[idx] = SYSTEM_PALLETE[frame_buffer.data[idx] as usize];
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

    fn play_sound(&mut self, sound: [Tone; 4]) {
        self.ch1_fr.set_value(sound[0].frequency);
        self.ch1_vo.set_value(sound[0].volume);
        self.ch1_duty.set_value(match sound[0].duty {
            WaveForm::Pulse12 => 0.125,
            WaveForm::Pulse25 => 0.25,
            WaveForm::Pulse50 => 0.50,
            WaveForm::Pusle75 => 0.75,
            WaveForm::Triangle => todo!(),
            WaveForm::Noise => todo!(),
        } as f64);

        self.ch2_fr.set_value(sound[1].frequency);
        self.ch2_vo.set_value(sound[1].volume);
        self.ch2_duty.set_value(match sound[1].duty {
            WaveForm::Pulse12 => 0.125,
            WaveForm::Pulse25 => 0.25,
            WaveForm::Pulse50 => 0.50,
            WaveForm::Pusle75 => 0.75,
            WaveForm::Triangle => todo!(),
            WaveForm::Noise => todo!(),
        } as f64);

        self.ch3_fr.set_value(sound[2].frequency);
        self.ch3_vo.set_value(sound[2].volume);

        // self.ch4_fr.set_value(sound[3].frequency);
        // self.ch4_vo.set_value(sound[3].volume);
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
    ch4_fr: Shared<f64>,
    ch4_vo: Shared<f64>,
) {
    spawn(move || {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap().config();
        let channel = config.channels as usize;

        let ch1_mono = ((var(&ch1_fr) | var(&ch1_duty)) >> pulse()) * var(&ch1_vo) * constant(0.25);
        let ch2_mono = ((var(&ch2_fr) | var(&ch2_duty)) >> pulse()) * var(&ch2_vo) * constant(0.25);

        let ch3_mono = (var(&ch3_fr) >> triangle()) * var(&ch3_vo) * constant(0.25);
        let ch4_mono = var(&ch4_vo) * constant(0.25);

        let pulse_mono = ch1_mono + ch2_mono;
        let tnd_mono = ch3_mono + ch4_mono;

        let mut total_mono = pulse_mono + tnd_mono;

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
