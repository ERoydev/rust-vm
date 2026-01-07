use crate::{bus::BusDevice, vm::VMWord};
use crate::error::{Result, VMError};

pub struct LinearMemory {
    bytes: Vec<u8>,
    size: usize,
}

impl LinearMemory {
    pub fn new(n: usize) -> Self {
        Self {
            bytes: vec![0; n],
            size: n,
        }
    }
}

impl BusDevice for LinearMemory {
    fn read(&self, addr: VMWord) -> Option<u8> {
        self.bytes.get(addr as usize).copied()
    }

    fn write(&mut self, addr: VMWord, value: u8) -> Result<()> {
        let addr_idx: usize = usize::from(addr);
        if addr_idx < self.size {
            self.bytes[addr_idx] = value;
            Ok(())
        } else {
            Err(VMError::OutOfBounds)
        }
    }

    fn memory_range(&self) -> usize {
        self.size
    }
}