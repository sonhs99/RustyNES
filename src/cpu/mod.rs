mod opcode;

use bitflags::bitflags;
use libc_print::{libc_print, libc_println};

use crate::memory::{Bus, MemoryBus};
pub use opcode::OpCode;
use opcode::OPCODE_TABLE;

const STACK: u16 = 0x0100;

#[warn(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    NoneAddressing,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Status: u8 {
        const CARRY = 0b0000_0001;
        const ZERO  = 0b0000_0010;
        const INT   = 0b0000_0100;
        const DEC   = 0b0000_1000;
        const BRK   = 0b0001_0000;
        const BRK2  = 0b0010_0000;
        const OVF   = 0b0100_0000;
        const NEG   = 0b1000_0000;

        const DEFAULT = 0b0010_0100;
    }
}

pub struct Instruction {
    pub byte: u8,
    pub size: u8,
    pub mnemonic: &'static str,
    pub mode: AddressingMode,
    pub pc: u16,
}

pub struct Cpu2A03 {
    pub(crate) a: u8,
    pub(crate) x: u8,
    pub(crate) y: u8,
    pub(crate) sp: u8,
    pub(crate) pc: u16,
    pub(crate) status: Status,
}

#[derive(Debug, Clone, Copy)]
pub enum Assembly {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    // undocumented
    ALR,
    ANC,
    ARR,
    AXS,
    LAX,
    SAX,
    DCP,
    ISC,
    RLA,
    RRA,
    SLO,
    SRE,
    // illegal
    AHX,
    LAS,
    SHX,
    SHY,
    TAS,
    XAA,
    STP,
}

impl Cpu2A03 {
    pub const fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0,
            status: Status::DEFAULT,
        }
    }

    pub fn reset(&mut self, mmu: &MemoryBus) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.status = Status::DEFAULT;
        self.sp = 0xFD;
        self.pc = mmu.read_word(0xFFFC);
        // self.pc = 0xC000;
    }

    pub fn fetch(&mut self, mmu: &MemoryBus) -> u8 {
        mmu.read_byte(self.pc)
    }

    pub fn decode(&self, opcode: u8) -> OpCode {
        OPCODE_TABLE[opcode as usize]
    }

    pub fn execute(&mut self, mmu: &mut MemoryBus, instruction: OpCode) -> u8 {
        let (elapsed_cycle, is_branched) = match (instruction.execute)(self, mmu, instruction.mode)
        {
            Ok((elapsed_cycle, is_branched)) => (elapsed_cycle, is_branched),
            Err(_) => {
                panic!(
                    "[CPU] Illigal Instruction Detected: {:4}",
                    instruction.mnemonic
                );
            }
        };
        if !is_branched {
            self.pc += instruction.size as u16;
        }
        elapsed_cycle
    }

    pub(crate) fn push8(&mut self, mmu: &mut MemoryBus, value: u8) {
        mmu.write_byte(STACK + self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub(crate) fn pop8(&mut self, mmu: &mut MemoryBus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        mmu.read_byte(STACK + self.sp as u16)
    }

    pub(crate) fn push16(&mut self, mmu: &mut MemoryBus, value: u16) {
        self.push8(mmu, (value >> 8) as u8);
        self.push8(mmu, value as u8);
    }
    pub(crate) fn pop16(&mut self, mmu: &mut MemoryBus) -> u16 {
        let low = self.pop8(mmu) as u16;
        let high = self.pop8(mmu) as u16;
        high << 8 | low
    }

    pub fn nmi(&mut self, mmu: &mut MemoryBus) -> u8 {
        self.push16(mmu, self.pc);
        let mut flag = self.status.clone();
        flag.set(Status::BRK, false);
        flag.set(Status::BRK2, true);

        self.push8(mmu, flag.bits());
        self.status.insert(Status::INT);
        self.pc = mmu.read_word(0xFFFA);
        2
    }
}

impl Instruction {
    pub const fn new(
        byte: u8,
        size: u8,
        mnemonic: &'static str,
        mode: AddressingMode,
        pc: u16,
    ) -> Self {
        Self {
            byte,
            size,
            mnemonic,
            mode,
            pc,
        }
    }
}

fn page_cross(addr1: u16, addr2: u16) -> bool {
    (addr1 & 0xFF00) != (addr2 & 0xFF00)
}

impl AddressingMode {
    pub fn fetch_addr(&self, cpu: &Cpu2A03, mmu: &MemoryBus) -> (u16, bool) {
        match self {
            AddressingMode::Immediate => (cpu.pc + 1, false),
            AddressingMode::ZeroPage => (mmu.read_byte(cpu.pc + 1) as u16, false),
            AddressingMode::ZeroPageX => {
                let res = mmu.read_byte(cpu.pc + 1).overflowing_add(cpu.x);
                (res.0 as u16, res.1)
            }
            AddressingMode::ZeroPageY => {
                let res = mmu.read_byte(cpu.pc + 1).overflowing_add(cpu.y);
                (res.0 as u16, res.1)
            }
            AddressingMode::Absolute => (mmu.read_word(cpu.pc + 1), false),
            AddressingMode::AbsoluteX => {
                let base = mmu.read_word(cpu.pc + 1);
                let addr = base.wrapping_add(cpu.x as u16);
                (addr, page_cross(base, addr))
            }
            AddressingMode::AbsoluteY => {
                let base = mmu.read_word(cpu.pc + 1);
                let addr = base.wrapping_add(cpu.y as u16);
                (addr, page_cross(base, addr))
            }
            AddressingMode::IndirectX => {
                let base = mmu.read_byte(cpu.pc + 1);
                let ptr = base.wrapping_add(cpu.x);
                let low = mmu.read_byte(ptr as u16) as u16;
                let high = mmu.read_byte(ptr.wrapping_add(1) as u16) as u16;
                (high << 8 | low, false)
            }
            AddressingMode::IndirectY => {
                let base = mmu.read_byte(cpu.pc + 1);
                let low = mmu.read_byte(base as u16) as u16;
                let high = mmu.read_byte(base.wrapping_add(1) as u16) as u16;
                let base = high << 8 | low;
                let addr = base.wrapping_add(cpu.y as u16);
                (addr, page_cross(base, addr))
            }
            AddressingMode::NoneAddressing => {
                panic!();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut mmu = MemoryBus::new();
        let mut cpu = Cpu2A03::new();
        cpu.pc = 0;
        for (idx, &value) in [0xa9, 0x05, 0x00].iter().enumerate() {
            mmu.write_byte(idx as u16, value as u8);
        }

        let opcode = cpu.fetch(&mmu);
        let instruction = cpu.decode(opcode);
        cpu.execute(&mut mmu, instruction);
        assert_eq!(cpu.a, 5);
        assert!(cpu.status.bits() & 0b0000_0010 == 0b00);
        assert!(cpu.status.bits() & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut mmu = MemoryBus::new();
        let mut cpu = Cpu2A03::new();
        cpu.pc = 0;
        cpu.a = 10;
        for (idx, &value) in [0xaa].iter().enumerate() {
            mmu.write_byte(idx as u16, value as u8);
        }

        let opcode = cpu.fetch(&mmu);
        let instruction = cpu.decode(opcode);
        cpu.execute(&mut mmu, instruction);
        assert_eq!(cpu.x, 10);
    }

    #[test]
    fn test_4_ops_working_together() {
        let mut mmu = MemoryBus::new();
        let mut cpu = Cpu2A03::new();
        cpu.pc = 0;
        for (idx, &value) in [0xa9, 0xc0, 0xaa, 0xe8].iter().enumerate() {
            mmu.write_byte(idx as u16, value as u8);
        }

        for _ in 0..3 {
            let opcode = cpu.fetch(&mmu);
            let instruction = cpu.decode(opcode);
            cpu.execute(&mut mmu, instruction);
        }
        assert_eq!(cpu.x, 0xc1);
    }

    #[test]
    fn test_inx_overflow() {
        let mut mmu = MemoryBus::new();
        let mut cpu = Cpu2A03::new();
        cpu.pc = 0;
        cpu.x = 0xFF;
        for (idx, &value) in [0xe8, 0xe8].iter().enumerate() {
            mmu.write_byte(idx as u16, value as u8);
        }

        for _ in 0..2 {
            let opcode = cpu.fetch(&mmu);
            let instruction = cpu.decode(opcode);
            cpu.execute(&mut mmu, instruction);
        }
        assert_eq!(cpu.x, 1);
    }

    #[test]
    fn test_lda_from_memory() {
        let mut mmu = MemoryBus::new();
        let mut cpu = Cpu2A03::new();
        cpu.pc = 0;
        for (idx, &value) in [0xa5, 0x10].iter().enumerate() {
            mmu.write_byte(idx as u16, value as u8);
        }
        mmu.write_byte(0x10, 0x55);

        let opcode = cpu.fetch(&mmu);
        let instruction = cpu.decode(opcode);
        cpu.execute(&mut mmu, instruction);
        assert_eq!(cpu.a, 0x55);
    }

    #[test]
    fn test_push_pop_ops() {
        let mut mmu = MemoryBus::new();
        let mut cpu = Cpu2A03::new();
        cpu.pc = 0;
        cpu.a = 20;
        cpu.sp = 0x80;
        for (idx, &value) in [0x48, 0xA9, 0x30, 0x68].iter().enumerate() {
            mmu.write_byte(idx as u16, value as u8);
        }

        for _ in 0..3 {
            let opcode = cpu.fetch(&mmu);
            let instruction = cpu.decode(opcode);
            cpu.execute(&mut mmu, instruction);
        }
        assert_eq!(cpu.a, 20);
        assert_eq!(cpu.sp, 0x80);
    }

    #[test]
    fn test_jsr_rts_ops() {
        let mut mmu = MemoryBus::new();
        let mut cpu = Cpu2A03::new();
        cpu.pc = 0;
        cpu.sp = 0x80;
        for (idx, &value) in [0x20, 0x07, 0x00, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA, 0x60]
            .iter()
            .enumerate()
        {
            mmu.write_byte(idx as u16, value as u8);
        }

        for _ in 0..3 {
            let opcode = cpu.fetch(&mmu);
            let instruction = cpu.decode(opcode);
            cpu.execute(&mut mmu, instruction);
        }
        assert_eq!(cpu.pc, 3);
    }
}
