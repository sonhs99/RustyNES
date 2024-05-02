use core::cell::RefCell;

use alloc::rc::Rc;

use crate::{joypad::JoypadButton, ppu::frame::Frame, Tone};

pub trait Hardware {
    fn is_active(&mut self) -> bool;
    fn draw_framebuffer(&mut self, frame_buffer: &Frame);
    fn pad_p1(&mut self) -> JoypadButton;
    fn pad_p2(&mut self) -> JoypadButton;
    fn play_sound(&mut self, sound: [Tone; 4]);
}

pub struct HardwareHandle(Rc<RefCell<dyn Hardware>>);

impl HardwareHandle {
    pub fn new<T: Hardware + 'static>(inner: T) -> Self {
        Self(Rc::new(RefCell::new(inner)))
    }

    pub fn get(&self) -> &Rc<RefCell<dyn Hardware>> {
        &self.0
    }
}
