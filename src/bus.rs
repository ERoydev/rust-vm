use crate::error::Result;
use crate::vm::VMWord;

// Interface for read and write access to memory or devices at specific addresses
pub trait BusDevice {
    fn read(&self, addr: VMWord) -> Option<u8>;
    fn write(&mut self, addr: VMWord, value: u8) -> Result<()>;
    fn memory_range(&self) -> usize;

    fn read2(&self, addr: VMWord) -> Option<u16> {
        if let Some(x0) = self.read(addr) {
            if let Some(x1) = self.read(addr + 1) {
                return Some((x0 as u16) | ((x1 as u16) << 8));
            }
        };
        None
    }
    fn write2(&mut self, addr: VMWord, value: u16) -> Result<()> {
        let low_byte = value & 0xff;
        let high_byte = (value & 0xff00) >> 8;

        // If the first write fails the second is not attempted, and the result is false, so called circuit
        self.write(addr, low_byte as u8)?;
        self.write(addr + 1, high_byte as u8)?;

        // ===== LOGGING
        println!("Write on Addr: {}, Value: {}", addr, low_byte);
        println!("Write on Addr: {}, Value: {}", addr + 1, high_byte);

        let proba = self.read2(addr).unwrap();
        println!("Result on Addr: {}, Value: {}\n", addr, proba);
        Ok(())
    }

    fn copy(&mut self, from: u16, to: u16, n: u16) -> bool {
        // So from and to are addresses, each address points to one byte in the memory -> [u8; 5000]
        // So in terms of that `n` represents how many bytes i want to copy
        for i in 0..n {
            if let Some(x) = self.read(from + i) {
                if let Err(err) = self.write(to + i, x) {
                    eprintln!("Memory error: {}", err.message());
                    return false;
                }
            } else {
                return false;
            }
        }
        return true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{Result, VMError};

    struct MockBus {
        memory: [u8; 1024],
    }

    impl MockBus {
        fn new() -> Self {
            Self { memory: [0; 1024] }
        }
    }

    impl BusDevice for MockBus {
        fn read(&self, addr: VMWord) -> Option<u8> {
            self.memory.get(addr as usize).copied()
        }
        fn write(&mut self, addr: VMWord, value: u8) -> Result<()> {
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
