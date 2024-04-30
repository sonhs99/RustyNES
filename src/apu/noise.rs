use super::util::*;
use super::{CPU_CLOCK, LENGTH_COUNTER_TABLE};

pub struct Noise {
    halt: bool,
    envelope: Envelope,

    timer: u16,
    length: u8,
}
