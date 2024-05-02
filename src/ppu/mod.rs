use alloc::vec;
use alloc::vec::Vec;
use libc_print::libc_println;

use crate::{
    cartrige::Mirroring,
    device::IOHandler,
    memory::{Bus, MemoryBus, MemoryRead, MemoryWrite},
};

use self::{
    addr::AddressRegister, control::ControllRegister, frame::Frame, mask::MaskRegister,
    scroll::ScrollRegister, status::StatusRegister,
};

mod addr;
mod control;
pub mod frame;
mod mask;
mod scroll;
mod status;

pub struct Ppu {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub mirroring: Mirroring,

    pub(crate) addr_reg: AddressRegister,
    pub(crate) ctrl_reg: ControllRegister,
    pub(crate) mask_reg: MaskRegister,
    pub(crate) status_reg: StatusRegister,
    pub(crate) scroll_reg: ScrollRegister,
    pub(crate) oam_addr_reg: u8,

    internal_data_buf: u8,
    cycles: usize,
    scanline: u16,
    nmi_interrupt: bool,
    chr_ram: bool,
    dma_enable: bool,

    frame: Frame,
    frame_tick: bool,
    ignore_nmi: bool,
}

#[derive(Debug)]
pub struct Rect {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

impl Rect {
    pub const fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        Self { x1, y1, x2, y2 }
    }

    pub fn is_inside(&self, x: usize, y: usize) -> bool {
        x >= self.x1 && x < self.x2 && y >= self.y1 && y < self.y2
    }
}

impl Ppu {
    #[cfg(test)]
    pub fn new_empty_rom() -> Self {
        Ppu::new(vec![0; 2048], Mirroring::Horizontal, false)
    }

    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring, chr_ram: bool) -> Self {
        Self {
            chr_rom,
            palette_table: [0; 32],
            vram: [0; 2048],
            oam_data: [0; 256],
            mirroring,
            addr_reg: AddressRegister::new(),
            ctrl_reg: ControllRegister::new(),
            mask_reg: MaskRegister::new(),
            status_reg: StatusRegister::new(),
            scroll_reg: ScrollRegister::new(),
            oam_addr_reg: 0,
            internal_data_buf: 0,
            cycles: 0,
            scanline: 0,
            nmi_interrupt: false,
            frame: Frame::new(),
            chr_ram,
            dma_enable: false,
            frame_tick: false,
            ignore_nmi: false,
        }
    }

    fn read_data(&mut self) -> u8 {
        let addr = self.addr_reg.get();
        self.addr_reg.increment(self.ctrl_reg.vram_addr_increment());

        match addr {
            0..0x2000 => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                result
            }
            0x2000..0x3000 => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..0x3F00 => panic!("[PPU] Attempt to Unused Space"),
            0x3F00..0x4000 => {
                self.internal_data_buf = self.vram[self.mirror_vram_addr(addr - 0x1000) as usize];
                self.palette_table[((addr - 0x3F00) % 0x20) as usize]
            }
            _ => panic!("unexpected mirror addr {}", addr),
        }
    }

    fn write_data(&mut self, value: u8) {
        let addr = self.addr_reg.get();
        self.addr_reg.increment(self.ctrl_reg.vram_addr_increment());

        match addr {
            0..0x2000 => {
                if self.chr_ram {
                    self.chr_rom[addr as usize] = value;
                } else {
                    panic!("[PPU] Attempt to Write Read-Only Data");
                }
            }
            0x2000..0x3000 => self.vram[self.mirror_vram_addr(addr) as usize] = value,
            0x3000..0x3F00 => panic!("[PPU] Access to Unused Space"),
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
                let mirrored_addr = addr - 0x10;
                self.palette_table[(mirrored_addr - 0x3F00) as usize] = value;
                self.palette_table[(addr - 0x3F00) as usize] = value;
            }
            0x3F00 | 0x3F04 | 0x3F08 | 0x3F0C => {
                let mirrored_addr = addr + 0x10;
                self.palette_table[(mirrored_addr - 0x3F00) as usize] = value;
                self.palette_table[(addr - 0x3F00) as usize] = value;
            }
            0x3F00..0x4000 => self.palette_table[((addr - 0x3F00) % 0x20) as usize] = value,
            _ => panic!("unexpected mirror addr {}", addr),
        }
    }

    fn mirror_vram_addr(&self, address: u16) -> u16 {
        let mirrored_addr = address & 0x2FFF;
        let vram_idx = mirrored_addr - 0x2000;
        let named_table = vram_idx / 0x400;
        match (&self.mirroring, named_table) {
            (Mirroring::Vertical, 2) | (Mirroring::Vertical, 3) => vram_idx - 0x800,
            (Mirroring::Horizontal, 2) => vram_idx - 0x400,
            (Mirroring::Horizontal, 1) => vram_idx - 0x400,
            (Mirroring::Horizontal, 3) => vram_idx - 0x800,
            _ => vram_idx,
        }
    }

    pub fn step(&mut self, cpu_cycles: u16) -> bool {
        self.cycles += cpu_cycles as usize * 3;
        if self.cycles >= 341 {
            if self.is_sprite_0_hit(self.cycles) {
                self.status_reg.set_sprite_0_hit(true);
            }
            self.scanline += 1;
            self.cycles = self.cycles - 341;

            if self.scanline == 241 {
                self.status_reg.set_vblank(!self.ignore_nmi);
                self.status_reg.set_sprite_0_hit(false);
                if self.ctrl_reg.generate_vblank_nmi() {
                    self.nmi_interrupt = !self.ignore_nmi;
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.status_reg.set_vblank(false);
                self.status_reg.set_sprite_0_hit(false);
                self.nmi_interrupt = false;
                self.ignore_nmi = false;
                if self.frame_tick
                    && (self.mask_reg.contains(MaskRegister::SPRITE)
                        || self.mask_reg.contains(MaskRegister::BACKGROUND))
                {
                    self.cycles += 1
                }
                self.frame_tick = !self.frame_tick;
                return true;
            }
        }
        false
    }

    pub fn render(&mut self) -> &Frame {
        let offset_x = self.scroll_reg.x as usize;
        let offset_y = self.scroll_reg.y as usize;
        let (first_bg, second_bg) = if self.ctrl_reg.nametable(self.mirroring) == 0 {
            (0, 0x400)
        } else {
            (0x400, 0)
        };

        for x in 0..frame::WIDTH {
            for y in 0..frame::HEIGHT {
                self.frame.set_pixel(x, y, self.palette_table[0]);
            }
        }

        if self.mask_reg.contains(MaskRegister::SPRITE) {
            self.render_sprite(false);
        }

        if self.mask_reg.contains(MaskRegister::BACKGROUND) {
            self.render_bg(
                first_bg,
                &Rect::new(offset_x, offset_y, frame::WIDTH, frame::HEIGHT),
                -(offset_x as isize),
                -(offset_y as isize),
            );

            if offset_x > 0 {
                self.render_bg(
                    second_bg,
                    &Rect::new(0, 0, offset_x, frame::HEIGHT),
                    (frame::WIDTH - offset_x) as isize,
                    0,
                );
            } else if offset_y > 0 {
                self.render_bg(
                    second_bg,
                    &Rect::new(0, 0, frame::WIDTH, offset_y),
                    0,
                    (frame::HEIGHT - offset_y) as isize,
                );
            }

            if self.mask_reg.contains(MaskRegister::SPRITE) {
                for x in 0..frame::WIDTH {
                    for y in 0..(self.oam_data[0] as usize + 8) {
                        self.frame.set_pixel(x, y, self.palette_table[0]);
                    }
                }
                self.render_bg(
                    0,
                    &Rect::new(0, 0, frame::WIDTH, self.oam_data[0] as usize + 8),
                    0,
                    0,
                );
            }
        }
        if self.mask_reg.contains(MaskRegister::SPRITE) {
            // self.render_sprite(false);
            self.render_sprite(true);
        }
        &self.frame
    }

    fn render_bg(&mut self, nametable_base: u16, range: &Rect, offset_x: isize, offset_y: isize) {
        let background_bank = self.ctrl_reg.background_pattern_addr();

        for i in 0..0x03C0 {
            let tile = self.vram[(nametable_base + i) as usize] as u16;
            let tile_x = i % 32;
            let tile_y = i / 32;
            let tile_base = (background_bank + tile * 16) as usize;
            let tile = &self.chr_rom[tile_base..=tile_base + 15];

            let palette = self.bg_palette(nametable_base, tile_x as usize, tile_y as usize);

            for y in 0..8 {
                let mut lower = tile[y];
                let mut upper = tile[y + 8];
                for x in (0..8).rev() {
                    let color_idx = (upper & 1) << 1 | (lower & 1);
                    upper = upper >> 1;
                    lower = lower >> 1;

                    let color = palette[color_idx as usize];
                    let pixel_x = tile_x as usize * 8 + x;
                    let pixel_y = tile_y as usize * 8 + y;

                    if color_idx != 0 && range.is_inside(pixel_x, pixel_y) {
                        self.frame.set_pixel(
                            (pixel_x as isize + offset_x) as usize,
                            (pixel_y as isize + offset_y) as usize,
                            color,
                        );
                    }
                }
            }
        }
    }

    fn render_sprite(&mut self, priority: bool) {
        let sprite_bank = self.ctrl_reg.sprite_pattern_addr();
        for sprite_idx in (0..self.oam_data.len()).step_by(4).rev() {
            let sprite_y = self.oam_data[sprite_idx];
            let tile_idx = self.oam_data[sprite_idx + 1] as u16;
            let sprite_attr = self.oam_data[sprite_idx + 2];
            let sprite_x = self.oam_data[sprite_idx + 3];
            if (sprite_attr & 0b0010_0000 == 0) == priority {
                let horizontal_flip = sprite_attr & 0b0100_0000 != 0;
                let vertical_flip = sprite_attr & 0b1000_0000 != 0;

                let tile_base = (tile_idx * 16 + sprite_bank) as usize;
                let tile = &self.chr_rom[tile_base..=tile_base + 15];

                let palette_idx = sprite_attr & 0b0000_0011;
                let palette = self.sprite_palette(palette_idx);
                for y in 0..8 {
                    let mut lower = tile[y];
                    let mut upper = tile[y + 8];
                    for x in (0..8).rev() {
                        let color_idx = (upper & 1) << 1 | (lower & 1);
                        upper = upper >> 1;
                        lower = lower >> 1;
                        if color_idx != 0 {
                            let color = palette[color_idx as usize];
                            self.frame.set_pixel(
                                sprite_x as usize + if horizontal_flip { 7 - x } else { x },
                                sprite_y as usize + if vertical_flip { 7 - y } else { y },
                                color,
                            )
                        }
                    }
                }
            }
        }
    }

    fn is_sprite_0_hit(&self, cycle: usize) -> bool {
        let y = self.oam_data[0] as u16;
        let x = self.oam_data[3] as u16;
        (y == self.scanline) && (x <= cycle as u16) && self.mask_reg.contains(MaskRegister::SPRITE)
    }

    pub fn nmi(&mut self) -> bool {
        let previous = self.nmi_interrupt;
        self.nmi_interrupt = false;
        previous
    }

    pub fn read_nmi(&self) -> bool {
        self.nmi_interrupt
    }

    pub fn read_status(&mut self) -> u8 {
        let data = self.status_reg.get();
        self.status_reg.set_vblank(false);
        self.addr_reg.reset_latch();
        self.scroll_reg.reset_latch();
        data
    }

    pub fn dma_enable(&mut self) -> bool {
        let previous = self.dma_enable;
        self.dma_enable = false;
        previous
    }

    fn bg_palette(&self, nametable_base: u16, tile_x: usize, tile_y: usize) -> [u8; 4] {
        let attr_base = (nametable_base + 0x03C0) as usize;
        let attr_idx = ((tile_x / 4) + (tile_y / 4) * 8) as usize;
        let attr_byte = self.vram[attr_base + attr_idx] as usize;

        let palette_idx = match (tile_y % 4 / 2, tile_x % 4 / 2) {
            (0, 0) => attr_byte & 0b11,
            (0, 1) => (attr_byte >> 2) & 0b11,
            (1, 0) => (attr_byte >> 4) & 0b11,
            (1, 1) => attr_byte >> 6,
            _ => panic!(),
        };
        let palette_start = (palette_idx as usize) * 4 + 1;
        [
            self.palette_table[0],
            self.palette_table[palette_start],
            self.palette_table[palette_start + 1],
            self.palette_table[palette_start + 2],
        ]
    }

    fn sprite_palette(&self, palette_idx: u8) -> [u8; 4] {
        let palette_start = 0x11 + (palette_idx * 4) as usize;
        [
            0,
            self.palette_table[palette_start],
            self.palette_table[palette_start + 1],
            self.palette_table[palette_start + 2],
        ]
    }

    pub fn cycle(&self) -> usize {
        self.cycles
    }

    pub fn scanline(&self) -> u16 {
        self.scanline
    }
}

impl IOHandler for Ppu {
    fn read(&mut self, mmu: &MemoryBus, address: u16) -> MemoryRead {
        if address >= 0x2000 && address < 0x4000 {
            match address % 8 {
                2 => {
                    // libc_println!("{}", self.scanline as usize * 341 + self.cycles);
                    let status = self.read_status();
                    match self.scanline as usize * 341 + self.cycles {
                        82181 | 82182 => {
                            // libc_println!("asdf: {:02X}", status);
                            self.ignore_nmi = true;
                            self.status_reg.set_vblank(false);
                            MemoryRead::Value(status | 0b1000_0000)
                        }
                        82180 => {
                            // libc_println!("fdsa: {:04X}", status);
                            self.ignore_nmi = true;
                            self.status_reg.set_vblank(false);
                            MemoryRead::Value(status & 0b0111_1111)
                        }
                        _ => MemoryRead::Value(status),
                    }
                }
                4 => MemoryRead::Value(self.oam_data[self.oam_addr_reg as usize]),
                7 => {
                    // let addr = self.addr_reg.get();
                    let value = self.read_data();
                    // libc_println!(
                    //     "[PPU] 0x{:04X} =   {:02X} [R]      C = {:02X}, S = {:02X}",
                    //     addr,
                    //     value,
                    //     self.ctrl_reg.bits(),
                    //     self.status_reg.get()
                    // );
                    MemoryRead::Value(value)
                }
                _ => panic!(
                    "Attempt to read write-only register: ${:04X} => ${:04X}",
                    address,
                    0x2000 + (address % 8)
                ),
            }
        } else {
            MemoryRead::Pass
        }
    }

    fn write(&mut self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite {
        if address >= 0x2000 && address < 0x4000 {
            match address % 8 {
                0 => {
                    // libc_println!(
                    //     "[PPU]      C =   {:02X} [W]              S = {:02X}",
                    //     value,
                    //     self.status_reg.get()
                    // );
                    let before_nmi = self.ctrl_reg.generate_vblank_nmi();
                    self.ctrl_reg.update(value);
                    if !before_nmi
                        && self.ctrl_reg.generate_vblank_nmi()
                        && self.status_reg.vblank()
                    {
                        self.nmi_interrupt = true;
                    }
                    MemoryWrite::Value(value)
                }
                1 => {
                    self.mask_reg.update(value);
                    MemoryWrite::Value(value)
                }
                3 => {
                    self.oam_addr_reg = value;
                    MemoryWrite::Value(value)
                }
                4 => {
                    self.oam_data[self.oam_addr_reg as usize] = value;
                    self.oam_addr_reg = self.oam_addr_reg.wrapping_add(1);
                    MemoryWrite::Value(value)
                }
                5 => {
                    self.scroll_reg.update(value);
                    // libc_println!(
                    //     "[PPU] Scroll = {:02X} => pos x:{:02X} y:{:02X} [W]",
                    //     value,
                    //     self.scroll_reg.x,
                    //     self.scroll_reg.y
                    // );
                    MemoryWrite::Value(value)
                }
                6 => {
                    self.addr_reg.update(value);
                    // libc_println!("[PPU]   Addr = {:04X}", self.addr_reg.get());
                    MemoryWrite::Value(value)
                }
                7 => {
                    // libc_println!(
                    //     "[PPU] 0x{:04X} =   {:02X} [W]      C = {:02X}, S = {:02X}",
                    //     self.addr_reg.get(),
                    //     value,
                    //     self.ctrl_reg.bits(),
                    //     self.status_reg.get()
                    // );
                    self.write_data(value);
                    MemoryWrite::Value(value)
                }
                _ => MemoryWrite::Block,
            }
        } else if address == 0x4014 {
            let base = (value as u16) << 8;
            for idx in 0..256 {
                self.oam_data[self.oam_addr_reg as usize] = mmu.read_byte(base + idx as u16);
                self.oam_addr_reg = self.oam_addr_reg.wrapping_add(1);
            }
            self.dma_enable = true;
            MemoryWrite::Value(value)
        } else {
            MemoryWrite::Pass
        }
    }
}

#[cfg(test)]
mod test {
    use crate::device::Device;

    use super::*;

    #[test]
    fn test_ppu_vram_writes() {
        let mut ppu = Ppu::new_empty_rom();
        ppu.addr_reg.update(0x23);
        ppu.addr_reg.update(0x05);
        ppu.write_data(0x66);

        assert_eq!(ppu.vram[0x0305], 0x66);
    }

    #[test]
    fn test_ppu_vram_reads() {
        let mut ppu = Ppu::new_empty_rom();
        ppu.ctrl_reg.update(0);
        ppu.vram[0x0305] = 0x66;

        ppu.addr_reg.update(0x23);
        ppu.addr_reg.update(0x05);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.addr_reg.get(), 0x2306);
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_ppu_vram_reads_cross_page() {
        let mut ppu = Ppu::new_empty_rom();
        ppu.ctrl_reg.update(0);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x0200] = 0x77;

        ppu.addr_reg.update(0x21);
        ppu.addr_reg.update(0xff);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
        assert_eq!(ppu.read_data(), 0x77);
    }

    #[test]
    fn test_ppu_vram_reads_step_32() {
        let mut ppu = Ppu::new_empty_rom();
        ppu.ctrl_reg.update(0b100);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x01ff + 32] = 0x77;
        ppu.vram[0x01ff + 64] = 0x88;

        ppu.addr_reg.update(0x21);
        ppu.addr_reg.update(0xff);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
        assert_eq!(ppu.read_data(), 0x77);
        assert_eq!(ppu.read_data(), 0x88);
    }

    #[test]
    fn test_vram_horizontal_mirror() {
        let mut ppu = Ppu::new_empty_rom();
        ppu.addr_reg.update(0x24);
        ppu.addr_reg.update(0x05);

        ppu.write_data(0x66); //write to a

        ppu.addr_reg.update(0x28);
        ppu.addr_reg.update(0x05);

        ppu.write_data(0x77); //write to B

        ppu.addr_reg.update(0x20);
        ppu.addr_reg.update(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x66); //read from A

        ppu.addr_reg.update(0x2C);
        ppu.addr_reg.update(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x77); //read from b
    }

    #[test]
    fn test_vram_vertical_mirror() {
        let mut ppu = Ppu::new(vec![0; 2048], Mirroring::Vertical, false);

        ppu.addr_reg.update(0x20);
        ppu.addr_reg.update(0x05);

        ppu.write_data(0x66); //write to A

        ppu.addr_reg.update(0x2C);
        ppu.addr_reg.update(0x05);

        ppu.write_data(0x77); //write to b

        ppu.addr_reg.update(0x28);
        ppu.addr_reg.update(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x66); //read from a

        ppu.addr_reg.update(0x24);
        ppu.addr_reg.update(0x05);

        ppu.read_data(); //load into buffer
        assert_eq!(ppu.read_data(), 0x77); //read from B
    }

    #[test]
    fn test_read_status_resets_latch() {
        let mut ppu = Ppu::new_empty_rom();
        ppu.vram[0x0305] = 0x66;

        ppu.addr_reg.update(0x21);
        ppu.addr_reg.update(0x23);
        ppu.addr_reg.update(0x05);

        ppu.read_data(); //load_into_buffer
        assert_ne!(ppu.read_data(), 0x66);

        ppu.read_status();

        ppu.addr_reg.update(0x23);
        ppu.addr_reg.update(0x05);

        ppu.read_data(); //load_into_buffer
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_ppu_vram_mirroring() {
        let mut ppu = Ppu::new_empty_rom();
        ppu.ctrl_reg.update(0);
        ppu.vram[0x0305] = 0x66;

        ppu.addr_reg.update(0x63); //0x6305 -> 0x2305
        ppu.addr_reg.update(0x05);

        ppu.read_data(); //load into_buffer
        assert_eq!(ppu.read_data(), 0x66);
    }

    #[test]
    fn test_read_status_resets_vblank() {
        let mut ppu = Ppu::new_empty_rom();
        ppu.status_reg.set_vblank(true);

        let status = ppu.read_status();

        assert_eq!(status >> 7, 1);
        assert_eq!(ppu.status_reg.get() >> 7, 0);
    }

    #[test]
    fn test_oam_dma() {
        use crate::MemoryBus;
        let mut mmu = MemoryBus::new();
        let ppu = Device::new(Ppu::new_empty_rom());

        mmu.register((0x4014, 0x4014), ppu.handler());

        const DMA_START: u16 = 0x2300;
        const DMA_START_BYTE: u8 = 0x23;

        ppu.borrow_mut().oam_addr_reg = 0x00;

        let mut data = [0x66; 256];
        data[0] = 0x77;
        data[255] = 0x88;

        for (idx, &d) in data.iter().enumerate() {
            mmu.write_byte(DMA_START + idx as u16, d);
        }

        mmu.write_byte(0x4014, DMA_START_BYTE);

        for (data, memory) in data.iter().zip(ppu.borrow_mut().oam_data.iter()) {
            assert_eq!(data, memory);
        }
        assert_eq!(ppu.borrow_mut().oam_addr_reg, 0);
    }
}
