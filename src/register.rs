use crate::constants::{START_ADDRESS, VMWord};
use crate::error::{Result, VMError};
use std::collections::HashMap;
use wincode_derive::{SchemaWrite};

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
#[derive(Clone, Copy, Debug, SchemaWrite)]
#[repr(u8)]
pub enum RegisterId {
    RR0, // return value register
    RR1,
    RR2,
    RR3,
    RPC, // program counter, holds the address of the next ix to exec
    RIR, // holds current instruction being executed when VM fetches an ix from memory
    RIM, // holds immediate values
}

impl RegisterId {
    pub fn id(&self) -> u8 {
        *self as u8
    }
}

pub const MAX_REGS: usize = 8;

/// Registers should hold a copy of the value from memory, not a pointer, and not remove the value from memory.
#[derive(Clone, Copy, Debug, SchemaWrite)]
pub struct Register {
    pub id: RegisterId,
    pub value: VMWord, // Bytes that it holds taken from memory
}

impl Register {
    pub fn new(register_type: RegisterId, value: VMWord) -> Self {
        Self {
            id: register_type,
            value: value,
        }
    }

    // I have to increment twice because each memory block is one byte, while my machine is 16-bit, which means i should read 2 bytes at a time
    pub fn inc_program_counter(&mut self) -> Result<()> {
        // increments the address
        self.value = self.value.checked_add(2).ok_or(VMError::Overflow)?;
        Ok(())
    }
}

#[derive(Debug, SchemaWrite)]
pub struct RegisterBank {
    pub register_map: HashMap<u8, Register>,
}

impl RegisterBank {
    pub fn new() -> Self {
        let reg_hashmap: HashMap<u8, Register> = [
            (
                RegisterId::RR0.id(),
                Register {
                    id: RegisterId::RR0,
                    value: 0x00,
                },
            ),
            (
                RegisterId::RR1.id(),
                Register {
                    id: RegisterId::RR1,
                    value: 0x00,
                },
            ),
            (
                RegisterId::RR2.id(),
                Register {
                    id: RegisterId::RR2,
                    value: 0x00,
                },
            ),
            (
                RegisterId::RR3.id(),
                Register {
                    id: RegisterId::RR3,
                    value: 0x00,
                },
            ),
            (
                RegisterId::RPC.id(),
                Register {
                    id: RegisterId::RPC,
                    value: START_ADDRESS, // PC is on the address where the program first instruction is loaded in memory, VM should load programs at 0x100 in this case
                },
            ),
            (
                // Should hold the instruction itself as bytes
                RegisterId::RIR.id(),
                Register {
                    id: RegisterId::RIR,
                    value: 0x00,
                },
            ),
            (
                RegisterId::RIM.id(),
                Register {
                    id: RegisterId::RIM,
                    value: 0x00,
                },
            ),
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

    pub fn get_register_mut(&mut self, name: u8) -> Result<&mut Register> {
        if let Some(reg) = self.register_map.get_mut(&name) {
            Ok(reg)
        } else {
            Err(VMError::UnknownRegister)
        }
    }
}
