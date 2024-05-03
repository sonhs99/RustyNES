use core::cell::{Ref, RefCell, RefMut};

use alloc::rc::Rc;

use crate::{
    memory::{MemoryBus, MemoryHandler, MemoryRead, MemoryWrite},
    ppu::{PpuHandler, Tile, TileSize},
};

pub struct Device<T>(Rc<RefCell<T>>, bool);

pub struct DevHandler<T>(Rc<RefCell<T>>, bool);

impl<T> Device<T> {
    pub fn new(inner: T) -> Self {
        Self::inner(inner, false)
    }

    pub fn mediate(inner: T) -> Self {
        Self::inner(inner, true)
    }

    fn inner(inner: T, debug: bool) -> Self {
        Self(Rc::new(RefCell::new(inner)), debug)
    }

    pub fn borrow<'a>(&'a self) -> Ref<'a, T> {
        self.0.borrow()
    }

    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a, T> {
        self.0.borrow_mut()
    }
}

pub trait IOHandler {
    fn read(&mut self, mmu: &MemoryBus, address: u16) -> MemoryRead;
    fn write(&mut self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite;
}

impl<T: IOHandler> Device<T> {
    pub fn handler(&self) -> DevHandler<T> {
        DevHandler(self.0.clone(), self.1)
    }
}

impl<T: IOHandler> MemoryHandler for DevHandler<T> {
    fn read(&self, mmu: &MemoryBus, address: u16) -> MemoryRead {
        match self.0.try_borrow_mut() {
            Ok(mut inner) => inner.read(mmu, address),
            Err(_) => {
                if self.1 {
                    MemoryRead::Pass
                } else {
                    panic!()
                }
            }
        }
    }
    fn write(&self, mmu: &MemoryBus, address: u16, value: u8) -> MemoryWrite {
        match self.0.try_borrow_mut() {
            Ok(mut inner) => inner.write(mmu, address, value),
            Err(_) => {
                if self.1 {
                    MemoryWrite::Pass
                } else {
                    panic!()
                }
            }
        }
    }
}

impl<T: PpuHandler> PpuHandler for DevHandler<T> {
    fn tile(&self, idx: usize, size: TileSize) -> Tile {
        match self.0.try_borrow() {
            Ok(inner) => inner.tile(idx, size),
            Err(_) => panic!(),
        }
    }

    fn read(&self, address: u16) -> MemoryRead {
        match self.0.try_borrow() {
            Ok(inner) => inner.read(address),
            Err(_) => panic!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) -> MemoryWrite {
        match self.0.try_borrow_mut() {
            Ok(mut inner) => inner.write(address, value),
            Err(_) => panic!(),
        }
    }
}
