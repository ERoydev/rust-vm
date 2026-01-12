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
