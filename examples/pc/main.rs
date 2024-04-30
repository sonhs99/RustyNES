extern crate rustynes;
mod hardware;

use rustynes::Nes;
use rustynes::Rom;

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

fn read_rom(rom_name: &str) -> Vec<u8> {
    let file = File::open(rom_name).expect("File Not Found");
    let buffer = BufReader::new(file);
    let mut rom: Vec<u8> = Vec::new();
    for byte_or_error in buffer.bytes() {
        let byte = byte_or_error.unwrap();
        rom.push(byte);
    }
    rom
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("usage: rustynes [FileName]");
    }

    let hw = hardware::Hardware::new();

    let rom = read_rom(&args[1]);

    let mut nes = Nes::new(&rom, hw);
    while nes.step() {}
}
