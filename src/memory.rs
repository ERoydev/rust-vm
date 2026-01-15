use crate::bus::BusDevice;
use crate::constants::VmAddr;
use crate::error::{Result, VMError};

#[derive(Debug, Clone)]
pub struct LinearMemory {
    pub bytes: Vec<u8>, // mem
    pub size: usize,
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

    fn as_bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    fn get_specific_memory_location(&self, idx: usize) -> u16 {
        let low_byte = self.bytes[idx] as u16;
        let high_byte = self.bytes[idx + 1] as u16;
        (high_byte << 8) | low_byte
    }

    fn get_subset_of_memory(&self, start_addr: usize, end_addr: usize) -> Vec<u8> {
        // Returns a Vec<u8> containing the memory from start_addr to end_addr (inclusive)
        self.bytes[start_addr..end_addr].to_vec()
    }
}
