use crate::memory::Bus;

use super::{AddressingMode, Cpu2A03, Instruction, MemoryBus, Status};

type OpcodeFn = fn(&mut Cpu2A03, &mut MemoryBus, AddressingMode) -> Result<(u8, bool), ()>;

#[derive(Clone, Copy)]
pub struct OpCode {
    pub mnemonic: &'static str,
    pub size: u8,
    pub mode: AddressingMode,
    pub execute: OpcodeFn,
}

impl OpCode {
    pub const fn new(
        mnemonic: &'static str,
        size: u8,
        mode: AddressingMode,
        execute: OpcodeFn,
    ) -> Self {
        Self {
            mnemonic,
            size,
            mode,
            execute,
        }
    }

    pub fn decode(&self, opcode: u8, cpu: &Cpu2A03) -> Instruction {
        Instruction::new(opcode, self.size, self.mnemonic, self.mode, cpu.pc)
    }
}

pub static OPCODE_TABLE: [OpCode; 256] = [
    /* 001 */ OpCode::new("BRK", 1, AddressingMode::NoneAddressing, brk),
    /* 002 */ OpCode::new("ORA", 2, AddressingMode::IndirectX, ora),
    /* 003 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 004 */ OpCode::new("*SLO", 2, AddressingMode::IndirectX, udf),
    /* 005 */ OpCode::new("*NOP", 2, AddressingMode::ZeroPage, nop),
    /* 006 */ OpCode::new("ORA", 2, AddressingMode::ZeroPage, ora),
    /* 007 */ OpCode::new("ASL", 2, AddressingMode::ZeroPage, asl),
    /* 008 */ OpCode::new("*SLO", 2, AddressingMode::ZeroPage, udf),
    /* 009 */ OpCode::new("PHP", 1, AddressingMode::NoneAddressing, php),
    /* 010 */ OpCode::new("ORA", 2, AddressingMode::Immediate, ora),
    /* 011 */ OpCode::new("ASL", 1, AddressingMode::NoneAddressing, asl),
    /* 012 */ OpCode::new("*ANC", 2, AddressingMode::Immediate, udf),
    /* 013 */ OpCode::new("*NOP", 3, AddressingMode::Absolute, nop),
    /* 014 */ OpCode::new("ORA", 3, AddressingMode::Absolute, ora),
    /* 015 */ OpCode::new("ASL", 3, AddressingMode::Absolute, asl),
    /* 016 */ OpCode::new("*SLO", 3, AddressingMode::Absolute, udf),
    /* 017 */ OpCode::new("BPL", 2, AddressingMode::Immediate, bpl),
    /* 018 */ OpCode::new("ORA", 2, AddressingMode::IndirectY, ora),
    /* 019 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 020 */ OpCode::new("*SLO", 2, AddressingMode::IndirectY, udf),
    /* 021 */ OpCode::new("*NOP", 2, AddressingMode::ZeroPageX, nop),
    /* 022 */ OpCode::new("ORA", 2, AddressingMode::ZeroPageX, ora),
    /* 023 */ OpCode::new("ASL", 2, AddressingMode::ZeroPageX, asl),
    /* 024 */ OpCode::new("*SLO", 2, AddressingMode::ZeroPageX, udf),
    /* 025 */ OpCode::new("CLC", 1, AddressingMode::NoneAddressing, clc),
    /* 026 */ OpCode::new("ORA", 3, AddressingMode::AbsoluteY, ora),
    /* 027 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 028 */ OpCode::new("*SLO", 3, AddressingMode::AbsoluteY, udf),
    /* 029 */ OpCode::new("*NOP", 3, AddressingMode::AbsoluteX, nop),
    /* 030 */ OpCode::new("ORA", 3, AddressingMode::AbsoluteX, ora),
    /* 031 */ OpCode::new("ASL", 3, AddressingMode::AbsoluteX, asl),
    /* 032 */ OpCode::new("*SLO", 3, AddressingMode::AbsoluteX, udf),
    /* 033 */ OpCode::new("JSR", 3, AddressingMode::Immediate, jsr),
    /* 034 */ OpCode::new("AND", 2, AddressingMode::IndirectX, and),
    /* 035 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 036 */ OpCode::new("*RLA", 2, AddressingMode::IndirectX, udf),
    /* 037 */ OpCode::new("BIT", 2, AddressingMode::ZeroPage, bit),
    /* 038 */ OpCode::new("AND", 2, AddressingMode::ZeroPage, and),
    /* 039 */ OpCode::new("ROL", 2, AddressingMode::ZeroPage, rol),
    /* 040 */ OpCode::new("*RLA", 2, AddressingMode::ZeroPage, udf),
    /* 041 */ OpCode::new("PLP", 1, AddressingMode::NoneAddressing, plp),
    /* 042 */ OpCode::new("AND", 2, AddressingMode::Immediate, and),
    /* 043 */ OpCode::new("ROL", 1, AddressingMode::NoneAddressing, rol),
    /* 044 */ OpCode::new("*ANC", 2, AddressingMode::Immediate, udf),
    /* 045 */ OpCode::new("BIT", 3, AddressingMode::Absolute, bit),
    /* 046 */ OpCode::new("AND", 3, AddressingMode::Absolute, and),
    /* 047 */ OpCode::new("ROL", 3, AddressingMode::Absolute, rol),
    /* 048 */ OpCode::new("*RLA", 3, AddressingMode::Absolute, udf),
    /* 049 */ OpCode::new("BMI", 2, AddressingMode::Immediate, bmi),
    /* 050 */ OpCode::new("AND", 2, AddressingMode::IndirectY, and),
    /* 051 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 052 */ OpCode::new("*RLA", 2, AddressingMode::IndirectY, udf),
    /* 053 */ OpCode::new("*NOP", 2, AddressingMode::ZeroPageX, nop),
    /* 054 */ OpCode::new("AND", 2, AddressingMode::ZeroPageX, and),
    /* 055 */ OpCode::new("ROL", 2, AddressingMode::ZeroPageX, rol),
    /* 056 */ OpCode::new("*RLA", 2, AddressingMode::ZeroPageX, udf),
    /* 057 */ OpCode::new("SEC", 1, AddressingMode::NoneAddressing, sec),
    /* 058 */ OpCode::new("AND", 3, AddressingMode::AbsoluteY, and),
    /* 059 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 060 */ OpCode::new("*RLA", 3, AddressingMode::AbsoluteY, udf),
    /* 061 */ OpCode::new("*NOP", 3, AddressingMode::AbsoluteX, nop),
    /* 062 */ OpCode::new("AND", 3, AddressingMode::AbsoluteX, and),
    /* 063 */ OpCode::new("ROL", 3, AddressingMode::AbsoluteX, rol),
    /* 064 */ OpCode::new("*RLA", 3, AddressingMode::AbsoluteX, udf),
    /* 065 */ OpCode::new("RTI", 1, AddressingMode::NoneAddressing, rti),
    /* 066 */ OpCode::new("EOR", 2, AddressingMode::IndirectX, eor),
    /* 067 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 068 */ OpCode::new("*SRE", 2, AddressingMode::IndirectX, udf),
    /* 069 */ OpCode::new("*NOP", 2, AddressingMode::ZeroPage, nop),
    /* 070 */ OpCode::new("EOR", 2, AddressingMode::ZeroPage, eor),
    /* 071 */ OpCode::new("LSR", 2, AddressingMode::ZeroPage, lsr),
    /* 072 */ OpCode::new("*SRE", 2, AddressingMode::ZeroPage, udf),
    /* 073 */ OpCode::new("PHA", 1, AddressingMode::NoneAddressing, pha),
    /* 074 */ OpCode::new("EOR", 2, AddressingMode::Immediate, eor),
    /* 075 */ OpCode::new("LSR", 1, AddressingMode::NoneAddressing, lsr),
    /* 076 */ OpCode::new("*ALR", 2, AddressingMode::Immediate, udf),
    /* 077 */
    OpCode::new("JMP", 3, AddressingMode::Immediate, jmp), //AddressingMode that acts as Immidiate
    /* 078 */ OpCode::new("EOR", 3, AddressingMode::Absolute, eor),
    /* 079 */ OpCode::new("LSR", 3, AddressingMode::Absolute, lsr),
    /* 080 */ OpCode::new("*SRE", 3, AddressingMode::Absolute, udf),
    /* 081 */ OpCode::new("BVC", 2, AddressingMode::Immediate, bvc),
    /* 082 */ OpCode::new("EOR", 2, AddressingMode::IndirectY, eor),
    /* 083 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 084 */ OpCode::new("*SRE", 2, AddressingMode::IndirectY, udf),
    /* 085 */ OpCode::new("*NOP", 2, AddressingMode::ZeroPageX, nop),
    /* 086 */ OpCode::new("EOR", 2, AddressingMode::ZeroPageX, eor),
    /* 087 */ OpCode::new("LSR", 2, AddressingMode::ZeroPageX, lsr),
    /* 088 */ OpCode::new("*SRE", 2, AddressingMode::ZeroPageX, udf),
    /* 089 */ OpCode::new("CLI", 1, AddressingMode::NoneAddressing, cli),
    /* 090 */ OpCode::new("EOR", 3, AddressingMode::AbsoluteY, eor),
    /* 091 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 092 */ OpCode::new("*SRE", 3, AddressingMode::AbsoluteY, udf),
    /* 093 */ OpCode::new("*NOP", 3, AddressingMode::AbsoluteX, nop),
    /* 094 */ OpCode::new("EOR", 3, AddressingMode::AbsoluteX, eor),
    /* 095 */ OpCode::new("LSR", 3, AddressingMode::AbsoluteX, lsr),
    /* 096 */ OpCode::new("*SRE", 3, AddressingMode::AbsoluteX, udf),
    /* 097 */ OpCode::new("RTS", 1, AddressingMode::NoneAddressing, rts),
    /* 098 */ OpCode::new("ADC", 2, AddressingMode::IndirectX, adc),
    /* 099 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 100 */ OpCode::new("*RRA", 2, AddressingMode::IndirectX, udf),
    /* 101 */ OpCode::new("*NOP", 2, AddressingMode::ZeroPage, nop),
    /* 102 */ OpCode::new("ADC", 2, AddressingMode::ZeroPage, adc),
    /* 103 */ OpCode::new("ROR", 2, AddressingMode::ZeroPage, ror),
    /* 104 */ OpCode::new("*RRA", 2, AddressingMode::ZeroPage, udf),
    /* 105 */ OpCode::new("PLA", 1, AddressingMode::NoneAddressing, pla),
    /* 106 */ OpCode::new("ADC", 2, AddressingMode::Immediate, adc),
    /* 107 */ OpCode::new("ROR", 1, AddressingMode::NoneAddressing, ror),
    /* 108 */ OpCode::new("*ARR", 2, AddressingMode::Immediate, udf),
    /* 109 */
    OpCode::new("JMP", 3, AddressingMode::Absolute, jmp), //AddressingMode:Indirect with 6502 bug
    /* 110 */ OpCode::new("ADC", 3, AddressingMode::Absolute, adc),
    /* 111 */ OpCode::new("ROR", 3, AddressingMode::Absolute, ror),
    /* 112 */ OpCode::new("*RRA", 3, AddressingMode::Absolute, udf),
    /* 113 */ OpCode::new("BVS", 2, AddressingMode::Immediate, bvs),
    /* 114 */ OpCode::new("ADC", 2, AddressingMode::IndirectY, adc),
    /* 115 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 116 */ OpCode::new("*RRA", 2, AddressingMode::IndirectY, udf),
    /* 117 */ OpCode::new("*NOP", 2, AddressingMode::ZeroPageX, nop),
    /* 118 */ OpCode::new("ADC", 2, AddressingMode::ZeroPageX, adc),
    /* 119 */ OpCode::new("ROR", 2, AddressingMode::ZeroPageX, ror),
    /* 120 */ OpCode::new("*RRA", 2, AddressingMode::ZeroPageX, udf),
    /* 121 */ OpCode::new("SEI", 1, AddressingMode::NoneAddressing, sei),
    /* 122 */ OpCode::new("ADC", 3, AddressingMode::AbsoluteY, adc),
    /* 123 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 124 */ OpCode::new("*RRA", 3, AddressingMode::AbsoluteY, udf),
    /* 125 */ OpCode::new("*NOP", 3, AddressingMode::AbsoluteX, nop),
    /* 126 */ OpCode::new("ADC", 3, AddressingMode::AbsoluteX, adc),
    /* 127 */ OpCode::new("ROR", 3, AddressingMode::AbsoluteX, ror),
    /* 128 */ OpCode::new("*RRA", 3, AddressingMode::AbsoluteX, udf),
    /* 129 */ OpCode::new("*NOP", 2, AddressingMode::Immediate, nop),
    /* 130 */ OpCode::new("STA", 2, AddressingMode::IndirectX, sta),
    /* 131 */ OpCode::new("*NOP", 2, AddressingMode::Immediate, nop),
    /* 132 */ OpCode::new("*SAX", 2, AddressingMode::IndirectX, udf),
    /* 133 */ OpCode::new("STY", 2, AddressingMode::ZeroPage, sty),
    /* 134 */ OpCode::new("STA", 2, AddressingMode::ZeroPage, sta),
    /* 135 */ OpCode::new("STX", 2, AddressingMode::ZeroPage, stx),
    /* 136 */ OpCode::new("*SAX", 2, AddressingMode::ZeroPage, udf),
    /* 137 */ OpCode::new("DEY", 1, AddressingMode::NoneAddressing, dey),
    /* 138 */ OpCode::new("*NOP", 2, AddressingMode::Immediate, nop),
    /* 139 */ OpCode::new("TXA", 1, AddressingMode::NoneAddressing, txa),
    /* 140 */
    OpCode::new("*XAA", 2, AddressingMode::Immediate, udf), //todo: highly unstable and not used
    /* 141 */ OpCode::new("STY", 3, AddressingMode::Absolute, sty),
    /* 142 */ OpCode::new("STA", 3, AddressingMode::Absolute, sta),
    /* 143 */ OpCode::new("STX", 3, AddressingMode::Absolute, stx),
    /* 144 */ OpCode::new("*SAX", 3, AddressingMode::Absolute, udf),
    /* 145 */ OpCode::new("BCC", 2, AddressingMode::Immediate, bcc),
    /* 146 */ OpCode::new("STA", 2, AddressingMode::IndirectY, sta),
    /* 147 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 148 */
    OpCode::new("*AHX", 2, AddressingMode::IndirectY, udf), //todo: highly unstable and not used
    /* 149 */ OpCode::new("STY", 2, AddressingMode::ZeroPageX, sty),
    /* 150 */ OpCode::new("STA", 2, AddressingMode::ZeroPageX, sta),
    /* 151 */ OpCode::new("STX", 2, AddressingMode::ZeroPageY, stx),
    /* 152 */ OpCode::new("*SAX", 2, AddressingMode::ZeroPageY, udf),
    /* 153 */ OpCode::new("TYA", 1, AddressingMode::NoneAddressing, tya),
    /* 154 */ OpCode::new("STA", 3, AddressingMode::AbsoluteY, sta),
    /* 155 */ OpCode::new("TXS", 1, AddressingMode::NoneAddressing, txs),
    /* 156 */
    OpCode::new("*TAS", 3, AddressingMode::AbsoluteY, udf), //todo: highly unstable and not used
    /* 157 */
    OpCode::new("*SHY", 3, AddressingMode::AbsoluteX, udf), //todo: highly unstable and not used
    /* 158 */ OpCode::new("STA", 3, AddressingMode::AbsoluteX, sta),
    /* 159 */
    OpCode::new("*SHX", 3, AddressingMode::AbsoluteY, udf), //todo: highly unstable and not used
    /* 160 */
    OpCode::new("*AHX", 3, AddressingMode::AbsoluteY, udf), //todo: highly unstable and not used
    /* 161 */ OpCode::new("LDY", 2, AddressingMode::Immediate, ldy),
    /* 162 */ OpCode::new("LDA", 2, AddressingMode::IndirectX, lda),
    /* 163 */ OpCode::new("LDX", 2, AddressingMode::Immediate, ldx),
    /* 164 */ OpCode::new("*LAX", 2, AddressingMode::IndirectX, udf),
    /* 165 */ OpCode::new("LDY", 2, AddressingMode::ZeroPage, ldy),
    /* 166 */ OpCode::new("LDA", 2, AddressingMode::ZeroPage, lda),
    /* 167 */ OpCode::new("LDX", 2, AddressingMode::ZeroPage, ldx),
    /* 168 */ OpCode::new("*LAX", 2, AddressingMode::ZeroPage, udf),
    /* 169 */ OpCode::new("TAY", 1, AddressingMode::NoneAddressing, tay),
    /* 170 */ OpCode::new("LDA", 2, AddressingMode::Immediate, lda),
    /* 171 */ OpCode::new("TAX", 1, AddressingMode::NoneAddressing, tax),
    /* 172 */
    OpCode::new("*LXA", 2, AddressingMode::Immediate, udf), //todo: highly unstable and not used
    /* 173 */ OpCode::new("LDY", 3, AddressingMode::Absolute, ldy),
    /* 174 */ OpCode::new("LDA", 3, AddressingMode::Absolute, lda),
    /* 175 */ OpCode::new("LDX", 3, AddressingMode::Absolute, ldx),
    /* 176 */ OpCode::new("*LAX", 3, AddressingMode::Absolute, udf),
    /* 177 */ OpCode::new("BCS", 2, AddressingMode::Immediate, bcs),
    /* 178 */ OpCode::new("LDA", 2, AddressingMode::IndirectY, lda),
    /* 179 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 180 */ OpCode::new("*LAX", 2, AddressingMode::IndirectY, udf),
    /* 181 */ OpCode::new("LDY", 2, AddressingMode::ZeroPageX, ldy),
    /* 182 */ OpCode::new("LDA", 2, AddressingMode::ZeroPageX, lda),
    /* 183 */ OpCode::new("LDX", 2, AddressingMode::ZeroPageY, ldx),
    /* 184 */ OpCode::new("*LAX", 2, AddressingMode::ZeroPageY, udf),
    /* 185 */ OpCode::new("CLV", 1, AddressingMode::NoneAddressing, clv),
    /* 186 */ OpCode::new("LDA", 3, AddressingMode::AbsoluteY, lda),
    /* 187 */ OpCode::new("TSX", 1, AddressingMode::NoneAddressing, tsx),
    /* 188 */
    OpCode::new("*LAS", 3, AddressingMode::AbsoluteY, udf), //todo: highly unstable and not used
    /* 189 */ OpCode::new("LDY", 3, AddressingMode::AbsoluteX, ldy),
    /* 190 */ OpCode::new("LDA", 3, AddressingMode::AbsoluteX, lda),
    /* 191 */ OpCode::new("LDX", 3, AddressingMode::AbsoluteY, ldx),
    /* 192 */ OpCode::new("*LAX", 3, AddressingMode::AbsoluteY, udf),
    /* 193 */ OpCode::new("CPY", 2, AddressingMode::Immediate, cpy),
    /* 194 */ OpCode::new("CMP", 2, AddressingMode::IndirectX, cmp),
    /* 195 */ OpCode::new("*NOP", 2, AddressingMode::Immediate, nop),
    /* 196 */ OpCode::new("*DCP", 2, AddressingMode::IndirectX, udf),
    /* 197 */ OpCode::new("CPY", 2, AddressingMode::ZeroPage, cpy),
    /* 198 */ OpCode::new("CMP", 2, AddressingMode::ZeroPage, cmp),
    /* 199 */ OpCode::new("DEC", 2, AddressingMode::ZeroPage, dec),
    /* 200 */ OpCode::new("*DCP", 2, AddressingMode::ZeroPage, udf),
    /* 201 */ OpCode::new("INY", 1, AddressingMode::NoneAddressing, iny),
    /* 202 */ OpCode::new("CMP", 2, AddressingMode::Immediate, cmp),
    /* 203 */ OpCode::new("DEX", 1, AddressingMode::NoneAddressing, dex),
    /* 204 */ OpCode::new("*AXS", 2, AddressingMode::Immediate, udf),
    /* 205 */ OpCode::new("CPY", 3, AddressingMode::Absolute, cpy),
    /* 206 */ OpCode::new("CMP", 3, AddressingMode::Absolute, cmp),
    /* 207 */ OpCode::new("DEC", 3, AddressingMode::Absolute, dec),
    /* 208 */ OpCode::new("*DCP", 3, AddressingMode::Absolute, udf),
    /* 209 */ OpCode::new("BNE", 2, AddressingMode::Immediate, bne),
    /* 210 */ OpCode::new("CMP", 2, AddressingMode::IndirectY, cmp),
    /* 211 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 212 */ OpCode::new("*DCP", 2, AddressingMode::IndirectY, udf),
    /* 213 */ OpCode::new("*NOP", 2, AddressingMode::ZeroPageX, nop),
    /* 214 */ OpCode::new("CMP", 2, AddressingMode::ZeroPageX, cmp),
    /* 215 */ OpCode::new("DEC", 2, AddressingMode::ZeroPageX, dec),
    /* 216 */ OpCode::new("*DCP", 2, AddressingMode::ZeroPageX, udf),
    /* 217 */ OpCode::new("CLD", 1, AddressingMode::NoneAddressing, cld),
    /* 218 */ OpCode::new("CMP", 3, AddressingMode::AbsoluteY, cmp),
    /* 219 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 220 */ OpCode::new("*DCP", 3, AddressingMode::AbsoluteY, udf),
    /* 221 */ OpCode::new("*NOP", 3, AddressingMode::AbsoluteX, nop),
    /* 222 */ OpCode::new("CMP", 3, AddressingMode::AbsoluteX, cmp),
    /* 223 */ OpCode::new("DEC", 3, AddressingMode::AbsoluteX, dec),
    /* 224 */ OpCode::new("*DCP", 3, AddressingMode::AbsoluteX, udf),
    /* 225 */ OpCode::new("CPX", 2, AddressingMode::Immediate, cpx),
    /* 226 */ OpCode::new("SBC", 2, AddressingMode::IndirectX, sbc),
    /* 227 */ OpCode::new("*NOP", 2, AddressingMode::Immediate, nop),
    /* 228 */ OpCode::new("*ISB", 2, AddressingMode::IndirectX, udf),
    /* 229 */ OpCode::new("CPX", 2, AddressingMode::ZeroPage, cpx),
    /* 230 */ OpCode::new("SBC", 2, AddressingMode::ZeroPage, sbc),
    /* 231 */ OpCode::new("INC", 2, AddressingMode::ZeroPage, inc),
    /* 232 */ OpCode::new("*ISB", 2, AddressingMode::ZeroPage, udf),
    /* 233 */ OpCode::new("INX", 1, AddressingMode::NoneAddressing, inx),
    /* 234 */ OpCode::new("SBC", 2, AddressingMode::Immediate, sbc),
    /* 235 */ OpCode::new("NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 236 */ OpCode::new("*SBC", 2, AddressingMode::Immediate, udf),
    /* 237 */ OpCode::new("CPX", 3, AddressingMode::Absolute, cpx),
    /* 238 */ OpCode::new("SBC", 3, AddressingMode::Absolute, sbc),
    /* 239 */ OpCode::new("INC", 3, AddressingMode::Absolute, inc),
    /* 240 */ OpCode::new("*ISB", 3, AddressingMode::Absolute, udf),
    /* 241 */ OpCode::new("BEQ", 2, AddressingMode::Immediate, beq),
    /* 242 */ OpCode::new("SBC", 2, AddressingMode::IndirectY, sbc),
    /* 243 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 244 */ OpCode::new("*ISB", 2, AddressingMode::IndirectY, udf),
    /* 245 */ OpCode::new("*NOP", 2, AddressingMode::ZeroPageX, nop),
    /* 246 */ OpCode::new("SBC", 2, AddressingMode::ZeroPageX, sbc),
    /* 247 */ OpCode::new("INC", 2, AddressingMode::ZeroPageX, inc),
    /* 248 */ OpCode::new("*ISB", 2, AddressingMode::ZeroPageX, udf),
    /* 249 */ OpCode::new("SED", 1, AddressingMode::NoneAddressing, sed),
    /* 250 */ OpCode::new("SBC", 3, AddressingMode::AbsoluteY, sbc),
    /* 251 */ OpCode::new("*NOP", 1, AddressingMode::NoneAddressing, nop),
    /* 252 */ OpCode::new("*ISB", 3, AddressingMode::AbsoluteY, udf),
    /* 253 */ OpCode::new("*NOP", 3, AddressingMode::AbsoluteX, nop),
    /* 254 */ OpCode::new("SBC", 3, AddressingMode::AbsoluteX, sbc),
    /* 255 */ OpCode::new("INC", 3, AddressingMode::AbsoluteX, inc),
    /* 256 */ OpCode::new("*ISB", 3, AddressingMode::AbsoluteX, udf),
];

pub fn adc(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let carry = cpu.status.contains(Status::CARRY);
    let (ptr, page_crossed) = operend.fetch_addr(cpu, mmu);
    let rhs = mmu.read_byte(ptr);
    let res = cpu.a.carrying_add(rhs, carry);
    cpu.status.set(Status::CARRY, res.1);
    cpu.status.set(Status::ZERO, res.0 == 0);
    cpu.status
        .set(Status::OVF, (rhs ^ res.0) & (cpu.a ^ res.0) & 0x80 != 0);
    cpu.status.set(Status::NEG, res.0 & 0b1000_0000 != 0);
    cpu.a = res.0;
    Ok(match operend {
        AddressingMode::Immediate => (2, false),
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::ZeroPageX => (4, false),
        AddressingMode::Absolute => (4, false),
        AddressingMode::AbsoluteX => (4 + page_crossed as u8, false),
        AddressingMode::AbsoluteY => (4 + page_crossed as u8, false),
        AddressingMode::IndirectX => (6, false),
        AddressingMode::IndirectY => (5 + page_crossed as u8, false),
        _ => panic!(),
    })
}

pub fn and(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, page_crossed) = operend.fetch_addr(cpu, mmu);
    let rhs = mmu.read_byte(ptr);
    let res = cpu.a & rhs;
    cpu.status.set(Status::ZERO, res == 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    cpu.a = res;
    Ok(match operend {
        AddressingMode::Immediate => (2, false),
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::ZeroPageX => (4, false),
        AddressingMode::Absolute => (4, false),
        AddressingMode::AbsoluteX => (4 + page_crossed as u8, false),
        AddressingMode::AbsoluteY => (4 + page_crossed as u8, false),
        AddressingMode::IndirectX => (6, false),
        AddressingMode::IndirectY => (5 + page_crossed as u8, false),
        _ => panic!(),
    })
}

pub fn asl(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let rhs = if let AddressingMode::NoneAddressing = operend {
        cpu.a
    } else {
        let (ptr, _) = operend.fetch_addr(cpu, mmu);
        mmu.read_byte(ptr)
    };
    let res = rhs << 1;
    cpu.status.set(Status::CARRY, rhs & 0b1000_0000 != 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    cpu.status.set(Status::ZERO, res == 0);
    if let AddressingMode::NoneAddressing = operend {
        cpu.a = res;
    } else {
        let (ptr, _) = operend.fetch_addr(cpu, mmu);
        mmu.write_byte(ptr, res);
    };
    Ok(match operend {
        AddressingMode::NoneAddressing => (2, false),
        AddressingMode::ZeroPage => (5, false),
        AddressingMode::ZeroPageX => (6, false),
        AddressingMode::Absolute => (6, false),
        AddressingMode::AbsoluteX => (7, false),
        _ => panic!(),
    })
}

pub fn bcc(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let is_branched = !cpu.status.contains(Status::CARRY);
    let page_crossed = if is_branched {
        let addr = operend.fetch_addr(cpu, mmu).0;
        let addr = mmu.read_byte(addr);
        let res = cpu.pc.wrapping_add(addr as i8 as u16).wrapping_add(2);
        let page_crossed = res & 0xFF00 != cpu.pc.wrapping_add(2) & 0xFF00;
        cpu.pc = res;
        page_crossed
    } else {
        false
    };
    Ok((2 + (is_branched as u8) + (page_crossed as u8), is_branched))
}

pub fn bcs(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let is_branched = cpu.status.contains(Status::CARRY);
    let page_crossed = if is_branched {
        let addr = operend.fetch_addr(cpu, mmu).0;
        let addr = mmu.read_byte(addr);
        let res = cpu.pc.wrapping_add(addr as i8 as u16).wrapping_add(2);
        let page_crossed = res & 0xFF00 != cpu.pc.wrapping_add(2) & 0xFF00;
        cpu.pc = res;
        page_crossed
    } else {
        false
    };
    Ok((2 + (is_branched as u8) + (page_crossed as u8), is_branched))
}

pub fn beq(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let is_branched = cpu.status.contains(Status::ZERO);
    let page_crossed = if is_branched {
        let addr = operend.fetch_addr(cpu, mmu).0;
        let addr = mmu.read_byte(addr);
        let res = cpu.pc.wrapping_add(addr as i8 as u16).wrapping_add(2);
        let page_crossed = res & 0xFF00 != cpu.pc.wrapping_add(2) & 0xFF00;
        cpu.pc = res;
        page_crossed
    } else {
        false
    };
    Ok((2 + (is_branched as u8) + (page_crossed as u8), is_branched))
}

pub fn bit(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, _) = operend.fetch_addr(cpu, mmu);
    let rhs = mmu.read_byte(ptr);
    let res = rhs & cpu.a;
    cpu.status.set(Status::ZERO, res == 0);
    cpu.status.set(Status::OVF, rhs & 0b0100_0000 != 0);
    cpu.status.set(Status::NEG, rhs & 0b1000_0000 != 0);
    Ok(match operend {
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::Absolute => (4, false),
        _ => panic!(),
    })
}

pub fn bmi(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let is_branched = cpu.status.contains(Status::NEG);
    let page_crossed = if is_branched {
        let addr = operend.fetch_addr(cpu, mmu).0;
        let addr = mmu.read_byte(addr);
        let res = cpu.pc.wrapping_add(addr as i8 as u16).wrapping_add(2);
        let page_crossed = res & 0xFF00 != cpu.pc.wrapping_add(2) & 0xFF00;
        cpu.pc = res;
        page_crossed
    } else {
        false
    };
    Ok((2 + (is_branched as u8) + (page_crossed as u8), is_branched))
}

pub fn bne(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let is_branched = !cpu.status.contains(Status::ZERO);
    let page_crossed = if is_branched {
        let addr = operend.fetch_addr(cpu, mmu).0;
        let addr = mmu.read_byte(addr);
        let res = cpu.pc.wrapping_add(addr as i8 as u16).wrapping_add(2);
        let page_crossed = res & 0xFF00 != cpu.pc.wrapping_add(2) & 0xFF00;
        cpu.pc = res;
        page_crossed
    } else {
        false
    };
    Ok((2 + (is_branched as u8) + (page_crossed as u8), is_branched))
}

pub fn bpl(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let is_branched = !cpu.status.contains(Status::NEG);
    let page_crossed = if is_branched {
        let addr = operend.fetch_addr(cpu, mmu).0;
        let addr = mmu.read_byte(addr);
        let res = cpu.pc.wrapping_add(addr as i8 as u16).wrapping_add(2);
        let page_crossed = res & 0xFF00 != cpu.pc.wrapping_add(2) & 0xFF00;
        cpu.pc = res;
        page_crossed
    } else {
        false
    };
    Ok((2 + (is_branched as u8) + (page_crossed as u8), is_branched))
}

pub fn brk(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.status.set(Status::BRK, true);
    cpu.push8(mmu, cpu.status.bits());
    cpu.push16(mmu, cpu.pc);
    cpu.pc = mmu.read_word(0xFFFE);
    Ok((7, true))
}

pub fn bvc(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let is_branched = !cpu.status.contains(Status::OVF);
    let page_crossed = if is_branched {
        let addr = operend.fetch_addr(cpu, mmu).0;
        let addr = mmu.read_byte(addr);
        let res = cpu.pc.wrapping_add(addr as i8 as u16).wrapping_add(2);
        let page_crossed = res & 0xFF00 != cpu.pc.wrapping_add(2) & 0xFF00;
        cpu.pc = res;
        page_crossed
    } else {
        false
    };
    Ok((2 + (is_branched as u8) + (page_crossed as u8), is_branched))
}

pub fn bvs(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let is_branched = cpu.status.contains(Status::OVF);
    let page_crossed = if is_branched {
        let addr = operend.fetch_addr(cpu, mmu).0;
        let addr = mmu.read_byte(addr);
        let res = cpu.pc.wrapping_add(addr as i8 as u16).wrapping_add(2);
        let page_crossed = res & 0xFF00 != cpu.pc.wrapping_add(2) & 0xFF00;
        cpu.pc = res;
        page_crossed
    } else {
        false
    };
    Ok((2 + (is_branched as u8) + (page_crossed as u8), is_branched))
}

pub fn clc(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.status.set(Status::CARRY, false);
    Ok((2, false))
}

pub fn cld(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.status.set(Status::DEC, false);
    Ok((2, false))
}

pub fn cli(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.status.set(Status::INT, false);
    Ok((2, false))
}

pub fn clv(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.status.set(Status::OVF, false);
    Ok((2, false))
}

pub fn cmp(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, page_crossed) = operend.fetch_addr(cpu, mmu);
    let rhs = mmu.read_byte(ptr);
    let res = cpu.a.wrapping_sub(rhs);
    cpu.status.set(Status::CARRY, cpu.a >= rhs);
    cpu.status.set(Status::ZERO, res == 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    Ok(match operend {
        AddressingMode::Immediate => (2, false),
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::ZeroPageX => (4, false),
        AddressingMode::Absolute => (4, false),
        AddressingMode::AbsoluteX => (4 + page_crossed as u8, false),
        AddressingMode::AbsoluteY => (4 + page_crossed as u8, false),
        AddressingMode::IndirectX => (6, false),
        AddressingMode::IndirectY => (5 + page_crossed as u8, false),
        _ => panic!(),
    })
}

pub fn cpx(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, _) = operend.fetch_addr(cpu, mmu);
    let rhs = mmu.read_byte(ptr);
    let res = cpu.x.wrapping_sub(rhs);
    cpu.status.set(Status::CARRY, cpu.x >= rhs);
    cpu.status.set(Status::ZERO, res == 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    Ok(match operend {
        AddressingMode::Immediate => (2, false),
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::Absolute => (4, false),
        _ => panic!(),
    })
}

pub fn cpy(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, _) = operend.fetch_addr(cpu, mmu);
    let rhs = mmu.read_byte(ptr);
    let res = cpu.y.wrapping_sub(rhs);
    cpu.status.set(Status::CARRY, cpu.y >= rhs);
    cpu.status.set(Status::ZERO, res == 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    Ok(match operend {
        AddressingMode::Immediate => (2, false),
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::Absolute => (4, false),
        _ => panic!(),
    })
}

pub fn dec(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, _) = operend.fetch_addr(cpu, mmu);
    let rhs = mmu.read_byte(ptr);
    let res = rhs.wrapping_sub(1);
    cpu.status.set(Status::ZERO, res == 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    mmu.write_byte(ptr, res);
    Ok(match operend {
        AddressingMode::ZeroPage => (5, false),
        AddressingMode::ZeroPageX => (6, false),
        AddressingMode::Absolute => (6, false),
        AddressingMode::AbsoluteX => (7, false),
        _ => panic!(),
    })
}

pub fn dex(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let res = cpu.x.wrapping_sub(1);
    cpu.status.set(Status::ZERO, res == 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    cpu.x = res;
    Ok((2, false))
}

pub fn dey(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let res = cpu.y.wrapping_sub(1);
    cpu.status.set(Status::ZERO, res == 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    cpu.y = res;
    Ok((2, false))
}

pub fn eor(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, page_crossed) = operend.fetch_addr(cpu, mmu);
    let rhs = mmu.read_byte(ptr);
    let res = cpu.a ^ rhs;
    cpu.status.set(Status::ZERO, res == 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    cpu.a = res;
    Ok(match operend {
        AddressingMode::Immediate => (2, false),
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::ZeroPageX => (4, false),
        AddressingMode::Absolute => (4, false),
        AddressingMode::AbsoluteX => (4 + page_crossed as u8, false),
        AddressingMode::AbsoluteY => (4 + page_crossed as u8, false),
        AddressingMode::IndirectX => (6, false),
        AddressingMode::IndirectY => (5 + page_crossed as u8, false),
        _ => panic!(),
    })
}

pub fn inc(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, _) = operend.fetch_addr(cpu, mmu);
    let rhs = mmu.read_byte(ptr);
    let res = rhs.wrapping_add(1);
    cpu.status.set(Status::ZERO, res == 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    mmu.write_byte(ptr, res);
    Ok(match operend {
        AddressingMode::ZeroPage => (5, false),
        AddressingMode::ZeroPageX => (6, false),
        AddressingMode::Absolute => (6, false),
        AddressingMode::AbsoluteX => (7, false),
        _ => panic!(),
    })
}

pub fn inx(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let res = cpu.x.wrapping_add(1);
    cpu.status.set(Status::ZERO, res == 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    cpu.x = res;
    Ok((2, false))
}

pub fn iny(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let res = cpu.y.wrapping_add(1);
    cpu.status.set(Status::ZERO, res == 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    cpu.y = res;
    Ok((2, false))
}

pub fn jmp(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let addr = match operend {
        AddressingMode::Immediate => mmu.read_word(cpu.pc + 1),
        AddressingMode::Absolute => {
            let base = mmu.read_word(cpu.pc + 1);
            let low = mmu.read_byte(base as u16) as u16;
            let high = if base & 0xFF == 0xFF {
                mmu.read_byte(base & 0xFF00 as u16) as u16
            } else {
                mmu.read_byte(base.wrapping_add(1) as u16) as u16
            };
            high << 8 | low
        }
        _ => panic!(),
    };
    cpu.pc = addr;
    Ok(match operend {
        AddressingMode::Immediate => (3, true),
        AddressingMode::Absolute => (5, true),
        _ => panic!(),
    })
}

pub fn jsr(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let addr = operend.fetch_addr(cpu, mmu).0;
    let addr = mmu.read_word(addr);
    cpu.push16(mmu, cpu.pc + 2);
    cpu.pc = addr;
    Ok((6, true))
}

pub fn lda(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, page_crossed) = operend.fetch_addr(cpu, mmu);
    let rhs = mmu.read_byte(ptr);
    cpu.status.set(Status::ZERO, rhs == 0);
    cpu.status.set(Status::NEG, rhs & 0b1000_0000 != 0);
    cpu.a = rhs;
    Ok(match operend {
        AddressingMode::Immediate => (2, false),
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::ZeroPageX => (4, false),
        AddressingMode::Absolute => (4, false),
        AddressingMode::AbsoluteX => (4 + page_crossed as u8, false),
        AddressingMode::AbsoluteY => (4 + page_crossed as u8, false),
        AddressingMode::IndirectX => (6, false),
        AddressingMode::IndirectY => (5 + page_crossed as u8, false),
        _ => panic!(),
    })
}

pub fn ldx(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, page_crossed) = operend.fetch_addr(cpu, mmu);
    let rhs = mmu.read_byte(ptr);
    cpu.status.set(Status::ZERO, rhs == 0);
    cpu.status.set(Status::NEG, rhs & 0b1000_0000 != 0);
    cpu.x = rhs;
    Ok(match operend {
        AddressingMode::Immediate => (2, false),
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::ZeroPageY => (4, false),
        AddressingMode::Absolute => (4, false),
        AddressingMode::AbsoluteY => (4 + page_crossed as u8, false),
        _ => panic!(),
    })
}

pub fn ldy(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, page_crossed) = operend.fetch_addr(cpu, mmu);
    let rhs = mmu.read_byte(ptr);
    cpu.status.set(Status::ZERO, rhs == 0);
    cpu.status.set(Status::NEG, rhs & 0b1000_0000 != 0);
    cpu.y = rhs;
    Ok(match operend {
        AddressingMode::Immediate => (2, false),
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::ZeroPageX => (4, false),
        AddressingMode::Absolute => (4, false),
        AddressingMode::AbsoluteX => (4 + page_crossed as u8, false),
        _ => panic!(),
    })
}

pub fn lsr(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let rhs = if let AddressingMode::NoneAddressing = operend {
        cpu.a
    } else {
        let (ptr, _) = operend.fetch_addr(cpu, mmu);
        mmu.read_byte(ptr)
    };
    let res = rhs >> 1;
    cpu.status.set(Status::CARRY, rhs & 0b0000_0001 != 0);
    cpu.status.set(Status::ZERO, res == 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    if let AddressingMode::NoneAddressing = operend {
        cpu.a = res;
    } else {
        let (ptr, _) = operend.fetch_addr(cpu, mmu);
        mmu.write_byte(ptr, res);
    };

    Ok(match operend {
        AddressingMode::NoneAddressing => (2, false),
        AddressingMode::ZeroPage => (5, false),
        AddressingMode::ZeroPageX => (6, false),
        AddressingMode::Absolute => (6, false),
        AddressingMode::AbsoluteX => (7, false),
        _ => panic!(),
    })
}

pub fn nop(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let page_crossed = if let operend = AddressingMode::NoneAddressing {
        false
    } else {
        operend.fetch_addr(cpu, mmu).1
    };
    Ok(match operend {
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::ZeroPageX => (4, false),
        AddressingMode::Absolute => (4, false),
        AddressingMode::AbsoluteX => (4 + page_crossed as u8, false),
        AddressingMode::AbsoluteY => (4 + page_crossed as u8, false),
        AddressingMode::IndirectX => (6, false),
        AddressingMode::IndirectY => (5 + page_crossed as u8, false),
        _ => (2, false),
    })
    // Ok((2, false))
}

pub fn ora(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, page_crossed) = operend.fetch_addr(cpu, mmu);
    let rhs = mmu.read_byte(ptr);
    let res = cpu.a | rhs;
    cpu.status.set(Status::ZERO, res == 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    cpu.a = res;
    Ok(match operend {
        AddressingMode::Immediate => (2, false),
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::ZeroPageX => (4, false),
        AddressingMode::Absolute => (4, false),
        AddressingMode::AbsoluteX => (4 + page_crossed as u8, false),
        AddressingMode::AbsoluteY => (4 + page_crossed as u8, false),
        AddressingMode::IndirectX => (6, false),
        AddressingMode::IndirectY => (5 + page_crossed as u8, false),
        _ => panic!(),
    })
}

pub fn pha(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.push8(mmu, cpu.a);
    Ok((3, false))
}

pub fn php(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.push8(mmu, cpu.status.bits() | 0b0001_0000);
    Ok((3, false))
}

pub fn pla(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.a = cpu.pop8(mmu) | (cpu.status.bits() & 0b0001_0000);
    cpu.status.set(Status::ZERO, cpu.a == 0);
    cpu.status.set(Status::NEG, cpu.a & 0b1000_0000 != 0);
    Ok((4, false))
}

pub fn plp(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let status = cpu.pop8(mmu);
    cpu.status = Status::from_bits(status).unwrap();
    cpu.status.remove(Status::BRK);
    cpu.status.insert(Status::BRK2);
    Ok((4, false))
}

pub fn rol(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let rhs = if let AddressingMode::NoneAddressing = operend {
        cpu.a
    } else {
        let (ptr, _) = operend.fetch_addr(cpu, mmu);
        mmu.read_byte(ptr)
    };
    let res = rhs << 1 | (cpu.status.bits() & 0x01);
    cpu.status.set(Status::CARRY, rhs & 0b1000_0000 != 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    if let AddressingMode::NoneAddressing = operend {
        cpu.a = res;
        cpu.status.set(Status::ZERO, res == 0);
    } else {
        let (ptr, _) = operend.fetch_addr(cpu, mmu);
        mmu.write_byte(ptr, res);
    };
    Ok(match operend {
        AddressingMode::NoneAddressing => (2, false),
        AddressingMode::ZeroPage => (5, false),
        AddressingMode::ZeroPageX => (6, false),
        AddressingMode::Absolute => (6, false),
        AddressingMode::AbsoluteX => (7, false),
        _ => panic!(),
    })
}

pub fn ror(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let rhs = if let AddressingMode::NoneAddressing = operend {
        cpu.a
    } else {
        let (ptr, _) = operend.fetch_addr(cpu, mmu);
        mmu.read_byte(ptr)
    };
    let res = rhs >> 1 | (cpu.status.bits() & 0x01) << 7;
    cpu.status.set(Status::CARRY, rhs & 0b0000_0001 != 0);
    cpu.status.set(Status::NEG, res & 0b1000_0000 != 0);
    if let AddressingMode::NoneAddressing = operend {
        cpu.a = res;
        cpu.status.set(Status::ZERO, res == 0);
    } else {
        let (ptr, _) = operend.fetch_addr(cpu, mmu);
        mmu.write_byte(ptr, res);
    };
    Ok(match operend {
        AddressingMode::NoneAddressing => (2, false),
        AddressingMode::ZeroPage => (5, false),
        AddressingMode::ZeroPageX => (6, false),
        AddressingMode::Absolute => (6, false),
        AddressingMode::AbsoluteX => (7, false),
        _ => panic!(),
    })
}

pub fn rti(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.status = Status::from_bits_truncate(cpu.pop8(mmu) | 0x20);
    cpu.pc = cpu.pop16(mmu);
    Ok((6, true))
}

pub fn rts(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.pc = cpu.pop16(mmu) + 1;
    Ok((6, true))
}

pub fn sbc(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let carry = cpu.status.contains(Status::CARRY);
    let (ptr, page_crossed) = operend.fetch_addr(cpu, mmu);
    let rhs = mmu.read_byte(ptr);
    let res = cpu.a.carrying_add(!rhs, carry);
    cpu.status.set(Status::CARRY, res.1);
    cpu.status.set(Status::ZERO, res.0 == 0);
    cpu.status
        .set(Status::OVF, (!rhs ^ res.0) & (cpu.a ^ res.0) & 0x80 != 0);
    cpu.status.set(Status::NEG, res.0 & 0b1000_0000 != 0);
    cpu.a = res.0;
    Ok(match operend {
        AddressingMode::Immediate => (2, false),
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::ZeroPageX => (4, false),
        AddressingMode::Absolute => (4, false),
        AddressingMode::AbsoluteX => (4 + page_crossed as u8, false),
        AddressingMode::AbsoluteY => (4 + page_crossed as u8, false),
        AddressingMode::IndirectX => (6, false),
        AddressingMode::IndirectY => (5 + page_crossed as u8, false),
        _ => panic!(),
    })
}

pub fn sec(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.status.set(Status::CARRY, true);
    Ok((2, false))
}

pub fn sed(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.status.set(Status::DEC, true);
    Ok((2, false))
}

pub fn sei(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.status.set(Status::INT, true);
    Ok((2, false))
}

pub fn sta(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, _) = operend.fetch_addr(cpu, mmu);
    mmu.write_byte(ptr, cpu.a);
    Ok(match operend {
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::ZeroPageX => (4, false),
        AddressingMode::Absolute => (4, false),
        AddressingMode::AbsoluteX => (5, false),
        AddressingMode::AbsoluteY => (5, false),
        AddressingMode::IndirectX => (6, false),
        AddressingMode::IndirectY => (6, false),
        _ => panic!(),
    })
}

pub fn stx(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, page_crossed) = operend.fetch_addr(cpu, mmu);
    mmu.write_byte(ptr, cpu.x);
    Ok(match operend {
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::ZeroPageY => (4, false),
        AddressingMode::Absolute => (4, false),
        _ => panic!(),
    })
}

pub fn sty(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    let (ptr, page_crossed) = operend.fetch_addr(cpu, mmu);
    mmu.write_byte(ptr, cpu.y);
    Ok(match operend {
        AddressingMode::ZeroPage => (3, false),
        AddressingMode::ZeroPageX => (4, false),
        AddressingMode::Absolute => (4, false),
        _ => panic!(),
    })
}

pub fn tax(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.x = cpu.a;
    cpu.status.set(Status::ZERO, cpu.a == 0);
    cpu.status.set(Status::NEG, cpu.a & 0b1000_0000 != 0);
    Ok((2, false))
}

pub fn tay(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.y = cpu.a;
    cpu.status.set(Status::ZERO, cpu.a == 0);
    cpu.status.set(Status::NEG, cpu.a & 0b1000_0000 != 0);
    Ok((2, false))
}

pub fn tsx(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.x = cpu.sp;
    cpu.status.set(Status::ZERO, cpu.sp == 0);
    cpu.status.set(Status::NEG, cpu.sp & 0b1000_0000 != 0);
    Ok((2, false))
}

pub fn txa(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.a = cpu.x;
    cpu.status.set(Status::ZERO, cpu.x == 0);
    cpu.status.set(Status::NEG, cpu.x & 0b1000_0000 != 0);
    Ok((2, false))
}

pub fn txs(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.sp = cpu.x;
    Ok((2, false))
}

pub fn tya(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    cpu.a = cpu.y;
    cpu.status.set(Status::ZERO, cpu.y == 0);
    cpu.status.set(Status::NEG, cpu.y & 0b1000_0000 != 0);
    Ok((2, false))
}

pub fn udf(
    cpu: &mut Cpu2A03,
    mmu: &mut MemoryBus,
    operend: AddressingMode,
) -> Result<(u8, bool), ()> {
    Err(())
}
