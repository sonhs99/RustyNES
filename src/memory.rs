use alloc::vec;
use alloc::{rc::Rc, vec::Vec};
use hashbrown::HashMap;
use libc_print::libc_println;

pub trait Bus {
    fn read_byte(&self, address: u16) -> u8;
    fn read_word(&self, address: u16) -> u16;
    fn write_byte(&mut self, address: u16, value: u8);
    fn write_word(&mut self, address: u16, value: u16);
}

pub struct MemoryBus {
    memory: [u8; 0x10000],
    handlers: HashMap<u16, Vec<Rc<dyn MemoryHandler>>>,
}

impl MemoryBus {
    pub fn new() -> Self {
        Self {
            memory: [0u8; 0x10000],
            handlers: HashMap::new(),
        }
    }

    pub fn register<T>(&mut self, range: (u16, u16), handler: T)
    where
        T: MemoryHandler + 'static,
    {
        let handler = Rc::new(handler);
        for addr in range.0..=range.1 {
            if self.handlers.contains_key(&addr) {
                match self.handlers.get_mut(&addr) {
                    Some(vec) => vec.push(handler.clone()),
                    None => (),
                }
            } else {
                self.handlers.insert(addr, vec![handler.clone()]);
            }
        }
    }
}

impl Bus for MemoryBus {
    fn read_byte(&self, address: u16) -> u8 {
        if let Some(handlers) = self.handlers.get(&address) {
            for handler in handlers {
                match handler.read(self, address) {
                    MemoryRead::Value(value) => return value,
                    MemoryRead::Pass => {}
                }
            }
        }
        if address < 0x2000 {
            self.memory[(address & 0x07FF) as usize]
        } else {
            self.memory[address as usize]
        }
    }

    fn read_word(&self, address: u16) -> u16 {
        self.read_byte(address) as u16 | ((self.read_byte(address + 1) as u16) << 8)
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        if let Some(handlers) = self.handlers.get(&address) {
            for handler in handlers {
                match handler.write(self, address, value) {
                    MemoryWrite::Value(value) => {
                        self.memory[address as usize] = value;
                        return;
                    }
                    MemoryWrite::Pass => {}
                    MemoryWrite::Block => return,
                }
            }
        }
        if address < 0x2000 {
            self.memory[(address & 0x07FF) as usize] = value;
        } else {
            self.memory[address as usize] = value;
        }
    }

    fn write_word(&mut self, address: u16, value: u16) {
        self.write_byte(address, value as u8);
        self.write_byte(address + 1, (value >> 8) as u8);
    }
}

pub enum MemoryRead {
    Value(u8),
    Pass,
}

pub enum MemoryWrite {
    Value(u8),
    Pass,
    Block,
}

pub trait MemoryHandler {
    fn read(&self, mmu: &MemoryBus, address: u16) -> MemoryRead;
    fn write(&self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite;
}

#[cfg(test)]
mod test {
    use crate::cartrige::Mirroring;
    use crate::ppu::Ppu;
    use crate::Device;

    use super::*;

    #[test]
    fn test_mem_read_write_to_ram() {
        let mut bus = MemoryBus::new();
        bus.write_byte(0x01, 0x55);
        assert_eq!(bus.read_byte(0x01), 0x55);
    }

    #[test]
    fn test_mem_read_write_to_ppu() {
        let mut mmu = MemoryBus::new();

        let ppu = Device::new(Ppu::new(vec![0; 0x2000], Mirroring::Horizontal, false));

        mmu.register((0x2000, 0x3FFF), ppu.handler());
        mmu.register((0x4014, 0x4014), ppu.handler());

        mmu.write_byte(0x2000, 0xFE);
        assert_eq!(ppu.borrow().ctrl_reg.bits(), 0xFE);

        // mmu.write_byte(0x2001, 0xFE);
        // assert_eq!(ppu.borrow().ctrl_reg.bits(), 0xFE);

        mmu.write_byte(0x2003, 0xFE);
        assert_eq!(ppu.borrow().oam_addr_reg, 0xFE);

        mmu.write_byte(0x2004, 0xFE);
        assert_eq!(ppu.borrow().oam_data[0xFE], 0xFE);
        assert_eq!(ppu.borrow().oam_addr_reg, 0xFF);

        mmu.write_byte(0x2006, 0xFE);
        assert_eq!(ppu.borrow().addr_reg.get(), 0x3E00);
        mmu.write_byte(0x2006, 0xFE);
        assert_eq!(ppu.borrow().addr_reg.get(), 0x3EFE);
    }
}
