pub static START_ADDRESS: u16 = 0x100; // I use this as start address, so i will first 256 bytes reserved for Program Segment Prefix

// VM word is currently 16-bit since i build 16bit VM
pub type VMWord = u16;
pub type VmAddr = VMWord;
