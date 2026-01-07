use std::collections::HashMap;
use crate::error::{Result, VMError};
use crate::vm::VMWord;

/*
    Register is a slot for storing a single value on the CPU. Registers are like workbench of the CPU.
    For the CPU to work with a piece of data, it has to be in one of the registers.

    Programs work by loading values from memory into registers, calculating values into other registers, then storing the final results back in memory.

    Most of the registers are general-purpose, but a few have designated roles - like `RPC` is program counter, `RFLAGS` condition flags, etc.

    The general purpose registers can be used to perform any program calculations. 
    The program counter is an unsigned integer which is the address of the next instruction in memory to execute. 
    The condition flags tell us information about the previous calculation.

    R0 to R3 are general-purpose registers
*/
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum RegisterId {
    RR0,   // return value register
    RR1,
    RR2,
    RR3,
    RSP,   // Stack pointer (function calls, local vars) points to the top of the stack
    RPC,   // program counter, holds the address of the next ix to exec
    RBP,   // base pointer, used to ref the base of the current stack frame, aka Frame Pointer, it is read-only
    RFLAGS, // condition flags (zero, carry, overflow) used for comparisons and branching
}

pub const MAX_REGS: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct Register {
    pub id: RegisterId,
    pub value: VMWord,
}

pub struct RegisterBank {
    pub register_map: HashMap<u8, Register>
}

impl RegisterBank {
    pub fn new() -> Self {
        let reg_hashmap: HashMap<u8, Register> = [
            (RegisterId::RR0 as u8, Register { id: RegisterId::RR0, value: 0x00 }),
            (RegisterId::RR1 as u8, Register { id: RegisterId::RR1, value: 0x00 }),
            (RegisterId::RR2 as u8, Register { id: RegisterId::RR2, value: 0x00 }),
            (RegisterId::RR3 as u8, Register { id: RegisterId::RR3, value: 0x00 }),
            (RegisterId::RSP as u8, Register { id: RegisterId::RSP, value: 0x00 }),
            (RegisterId::RPC as u8, Register { id: RegisterId::RPC, value: 0x100 }), // PC is on the address where the program first instruction is loaded in memory.
            // VM should load programs at 0x100 in this case
            (RegisterId::RBP as u8, Register { id: RegisterId::RBP, value: 0x00 }),
            (RegisterId::RFLAGS as u8, Register { id: RegisterId::RFLAGS, value: 0x00 }),
        ]
        .into();

        Self {
            register_map: reg_hashmap,
        }
    }
    pub fn get_register_read_only(&self, name: u8) -> Result<Register> {
        if let Some(reg) = self.register_map.get(&name).copied() {
            Ok(reg)
        } else {
            Err(VMError::UnknownRegister)
        }
    }

    pub fn get_register_mut(&mut self, name: u8) -> Option<&mut Register> {
        self.register_map.get_mut(&name)
    }
}