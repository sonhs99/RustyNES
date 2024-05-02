#![no_std]
#![feature(bigint_helper_methods)]
#![feature(exclusive_range_pattern)]
#![feature(type_alias_impl_trait)]

extern crate alloc;

use alloc::vec::Vec;
use apu::Apu;
pub use cartrige::Rom;
use cpu::{Cpu2A03, Instruction};
use device::Device;
use hardware::HardwareHandle;
use joypad::Joypad;
use libc_print::{libc_print, libc_println};
use memory::MemoryBus;
use ppu::Ppu;

pub use apu::{Tone, WaveForm};
pub use hardware::Hardware;
pub use joypad::JoypadButton;
pub use ppu::frame::{Frame, HEIGHT, WIDTH};

use crate::memory::Bus;

mod apu;
mod cartrige;
mod cpu;
mod device;
mod hardware;
mod joypad;
mod memory;
mod ppu;

pub struct Nes {
    cpu: Cpu2A03,
    mmu: MemoryBus,
    rom: Device<Rom>,
    ppu: Device<Ppu>,
    apu: Device<Apu>,
    pad: Device<Joypad>,
    cycles: usize,

    hardware: HardwareHandle,
}

impl Nes {
    pub fn new<T>(raw: &Vec<u8>, hardware: T) -> Self
    where
        T: Hardware + 'static,
    {
        let mut cpu = Cpu2A03::new();
        let mut mmu = MemoryBus::new();

        let rom = Device::new(Rom::new(raw).unwrap());
        let ppu = Device::new(Ppu::new(
            rom.borrow().chr_rom.clone(),
            rom.borrow().mirroring,
            rom.borrow().chr_ram,
        ));
        let apu = Device::new(Apu::new());

        let pad = Device::new(Joypad::new());

        mmu.register((0x6000, 0xFFFF), rom.handler());

        mmu.register((0x2000, 0x3FFF), ppu.handler());
        mmu.register((0x4014, 0x4014), ppu.handler());

        mmu.register((0x4000, 0x4013), apu.handler());
        mmu.register((0x4015, 0x4015), apu.handler());
        mmu.register((0x4017, 0x4017), apu.handler());

        mmu.register((0x4016, 0x4017), pad.handler());

        cpu.reset(&mmu);

        Self {
            cpu,
            mmu,
            rom,
            ppu,
            apu,
            pad,
            cycles: 0,
            hardware: HardwareHandle::new(hardware),
        }
    }

    pub fn step(&mut self) -> bool {
        let elapsed_cycles = if self.ppu.borrow_mut().nmi() {
            // libc_println!("NMI Occured");
            self.cpu.nmi(&mut self.mmu)
        } else {
            let opcode = self.cpu.fetch(&self.mmu);
            let instruction = self.cpu.decode(opcode);
            // self.log(&instruction.decode(opcode, &self.cpu));
            self.cpu.execute(&mut self.mmu, instruction)
        };

        // let elapsed_cycles = if self.ppu.borrow_mut().dma_enable() {
        //     elapsed_cycles as u16 + 512
        // } else {
        //     elapsed_cycles as u16
        // };

        self.cycles += elapsed_cycles as usize;

        let before_nmi = self.ppu.borrow().read_nmi();
        self.ppu.borrow_mut().step(elapsed_cycles as u16);
        let after_nmi = self.ppu.borrow().read_nmi();

        let volume = self.apu.borrow_mut().step(elapsed_cycles as u16);
        if !before_nmi && after_nmi {
            let mut ppu = self.ppu.borrow_mut();
            let frame = ppu.render();
            self.hardware.get().borrow_mut().draw_framebuffer(frame);
        }
        self.hardware.get().borrow_mut().play_sound(volume);

        let new_p1_state = self.hardware.get().borrow_mut().pad_p1();
        let new_p2_state = self.hardware.get().borrow_mut().pad_p2();
        if (new_p1_state.bits() != self.pad.borrow().status[0].bits())
            || (new_p2_state.bits() != self.pad.borrow().status[1].bits())
        {
            self.pad.borrow_mut().update(new_p1_state, new_p2_state);
        }
        self.hardware.get().borrow_mut().is_active()
    }

    fn log(&mut self, instruction: &Instruction) {
        libc_print!("{:04X}  ", self.cpu.pc);
        for i in 0..3u16 {
            if i < instruction.size as u16 {
                libc_print!("{:02X} ", self.mmu.read_byte(instruction.pc + i));
            } else {
                libc_print!("   ");
            }
        }
        libc_print!("{:>4} ", instruction.mnemonic,);
        match instruction.mode {
            cpu::AddressingMode::Immediate => {
                if instruction.mnemonic == "JMP" {
                    libc_print!(
                        "${:02X}{:02X}                       ",
                        self.mmu.read_byte(instruction.pc + 2),
                        self.mmu.read_byte(instruction.pc + 1)
                    )
                } else {
                    libc_print!(
                        "#${:02X}                        ",
                        self.mmu.read_byte(instruction.pc + 1)
                    )
                }
            }
            cpu::AddressingMode::ZeroPage => {
                let addr = instruction.mode.fetch_addr(&self.cpu, &self.mmu).0;
                libc_print!(
                    "${:02X} = {:02X}                    ",
                    self.mmu.read_byte(instruction.pc + 1),
                    self.mmu.read_byte(addr),
                )
            }
            cpu::AddressingMode::ZeroPageX => {
                let addr = instruction.mode.fetch_addr(&self.cpu, &self.mmu).0;
                libc_print!(
                    "${:02X},X = {:02X}                  ",
                    self.mmu.read_byte(instruction.pc + 1),
                    self.mmu.read_byte(addr),
                )
            }
            cpu::AddressingMode::ZeroPageY => {
                let addr = instruction.mode.fetch_addr(&self.cpu, &self.mmu).0;
                libc_print!(
                    "${:02X},Y = {:02X}                  ",
                    self.mmu.read_byte(instruction.pc + 1),
                    self.mmu.read_byte(addr),
                )
            }
            cpu::AddressingMode::Absolute => {
                let addr = instruction.mode.fetch_addr(&self.cpu, &self.mmu).0;
                if addr >= 0x2000 {
                    libc_print!(
                        "${:02X}{:02X}                       ",
                        self.mmu.read_byte(instruction.pc + 2),
                        self.mmu.read_byte(instruction.pc + 1)
                    )
                } else {
                    libc_print!(
                        "${:02X}{:02X} = {:02X}                  ",
                        self.mmu.read_byte(instruction.pc + 2),
                        self.mmu.read_byte(instruction.pc + 1),
                        self.mmu.read_byte(addr)
                    )
                }
            }
            cpu::AddressingMode::AbsoluteX => {
                libc_print!(
                    "${:02X}{:02X},X                     ",
                    self.mmu.read_byte(instruction.pc + 2),
                    self.mmu.read_byte(instruction.pc + 1)
                )
            }
            cpu::AddressingMode::AbsoluteY => {
                libc_print!(
                    "${:02X}{:02X},Y                     ",
                    self.mmu.read_byte(instruction.pc + 2),
                    self.mmu.read_byte(instruction.pc + 1)
                )
            }
            cpu::AddressingMode::IndirectX => {
                libc_print!(
                    "(${:02X},X)                     ",
                    self.mmu.read_byte(instruction.pc + 1)
                )
            }
            cpu::AddressingMode::IndirectY => {
                libc_print!(
                    "(${:02X}),Y                     ",
                    self.mmu.read_byte(instruction.pc + 1)
                )
            }
            cpu::AddressingMode::NoneAddressing => {
                libc_print!("                            ")
            }
        }
        libc_print!(
            "A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:-3} SL:{:<3} [",
            self.cpu.a,
            self.cpu.x,
            self.cpu.y,
            self.cpu.status.bits(),
            self.cpu.sp,
            self.ppu.borrow().cycle(),
            self.ppu.borrow().scanline(),
        );
        let mut pos = 0x80;
        let bit = self.cpu.status.bits();

        for c in "NV*BDIZC".chars() {
            libc_print!("{}", if bit & pos != 0 { c } else { '-' });
            pos = pos >> 1;
        }
        libc_println!("]");
    }
}
