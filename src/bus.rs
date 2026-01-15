use crate::{
    constants::VmAddr,
    error::{Result, VMError},
};

// Interface for read and write access to memory or devices at specific addresses
pub trait BusDevice: std::fmt::Debug {
    fn read(&self, addr: VmAddr) -> Option<u8>;
    fn write(&mut self, addr: VmAddr, value: u8) -> Result<()>;
    fn memory_range(&self) -> usize;
    fn as_bytes(&self) -> &Vec<u8>;

    fn read2(&self, addr: VmAddr) -> Option<u16> {
        if let Some(x0) = self.read(addr) {
            if let Some(x1) = self.read(addr + 1) {
                return Some((x0 as u16) | ((x1 as u16) << 8));
            }
        };
        None
    }
    fn write2(&mut self, addr: VmAddr, value: u16) -> Result<()> {
        let low_byte = value & 0xff;
        let high_byte = (value & 0xff00) >> 8;

        // If the first write fails the second is not attempted, and the result is false, so called circuit
        self.write(addr, low_byte as u8)?;
        self.write(addr + 1, high_byte as u8)?;

        // ===== LOGGING
        println!("Write on Addr: {}, Value: {}", addr, low_byte);
        println!("Write on Addr: {}, Value: {}", addr + 1, high_byte);

        let read_written_addr = self.read2(addr).unwrap();
        println!("Result on Addr: {}, Value: {}\n", addr, read_written_addr);
        Ok(())
    }

    fn copy(&mut self, from_addr: VmAddr, to_addr: VmAddr) -> Result<()> {
        // So from and to are addresses, each address points to one byte in the memory -> [u8; 5000]
        // TODO: Maybe its better to pass whole Register object and access the value on that memory address by getter, instead of passing register address like that
        if let Some(bytes) = self.read2(from_addr) {
            if let Err(err) = self.write2(to_addr, bytes) {
                return Err(err);
            }
        } else {
            return Err(VMError::CopyInstructionFail);
        }

        Ok(())
    }

    fn get_specific_memory_location(&self, idx: usize) -> u16;
    fn get_subset_of_memory(&self, start_addr: usize, end_addr: usize) -> Vec<u8>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{Result, VMError};

    #[derive(Debug)]
    struct MockBus {
        memory: Vec<u8>,
    }

    impl MockBus {
        fn new() -> Self {
            Self {
                memory: vec![0; 1024],
            }
        }
    }

    impl BusDevice for MockBus {
        fn read(&self, addr: VmAddr) -> Option<u8> {
            self.memory.get(addr as usize).copied()
        }
        fn write(&mut self, addr: VmAddr, value: u8) -> Result<()> {
            if let Some(slot) = self.memory.get_mut(addr as usize) {
                *slot = value;
                Ok(())
            } else {
                Err(VMError::OutOfBounds)
            }
        }
        fn memory_range(&self) -> usize {
            self.memory.len()
        }

        fn as_bytes(&self) -> &Vec<u8> {
            &self.memory
        }

        fn get_specific_memory_location(&self, idx: usize) -> u16 {
            300
        }

        fn get_subset_of_memory(&self, start_addr: usize, end_addr: usize) -> Vec<u8> {
            vec![12, 23]
        }
    }

    #[test]
    fn test_write2_reads_back_correct_value() {
        let mut bus = MockBus::new();
        let addr = 10;
        let value: u16 = 0x3005;
        bus.write2(addr, value).unwrap();
        assert_eq!(bus.read2(addr), Some(value));
    }
}
