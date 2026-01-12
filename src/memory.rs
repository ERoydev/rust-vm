use crate::bus::BusDevice;
use crate::constants::VmAddr;
use crate::error::{Result, VMError};

#[derive(Debug)]
pub struct LinearMemory {
    pub bytes: Vec<u8>, // mem
    size: usize,
}

impl LinearMemory {
    // newMemory implementation
    pub fn new(n: usize) -> Self {
        Self {
            bytes: vec![0; n],
            size: n,
        }
    }
}

impl BusDevice for LinearMemory {
    fn read(&self, addr: VmAddr) -> Option<u8> {
        self.bytes.get(addr as usize).copied()
    }

    fn write(&mut self, addr: VmAddr, value: u8) -> Result<()> {
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
