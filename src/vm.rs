#![allow(dead_code)]

use crate::error::Result;
use crate::{
    bus::BusDevice,
    error::VMError,
    memory::LinearMemory,
    register::{Register, RegisterBank, RegisterId},
};

// VM word is currently 16-bit since i build 16bit VM
pub type VMWord = u16;

// The VM config
pub struct Config {}

// It will simulate the computer for the 16bit VM
pub struct VM {
    pub registers: RegisterBank,
    pub memory: Box<dyn BusDevice>, // main memory
    pub halted: bool, // Signal when the VM should stop processing instructions, after program finishes or encounter a fatal error
}

impl VM {
    pub fn new() -> Self {
        Self {
            registers: RegisterBank::new(),
            memory: Box::new(LinearMemory::new(0)),
            halted: false,
        }
    }

    pub fn set_memory(&mut self, memory: Box<dyn BusDevice>) {
        self.memory = memory;
        log::info!("Set a new memory");
    }

    pub fn step(&mut self) -> Result<()> {
        // When the VM runs, we going to run a program(binary) and the VM is going to call step on that program until its done until we have some kind of outcome
        match self.registers.get_register_read_only(RegisterId::RPC as u8) {
            Ok(reg) => {
                let pc = reg.value;
                let instruction = self.memory.read2(pc).unwrap();
                println!("{} @ {:?}", instruction, pc);
                Ok(())
            }
            Err(err) => Err(VMError::UnknownRegister),
        }
    }

    pub fn halt(&mut self) {
        self.halted = true;
    }

    pub fn read(&mut self, source_reg: Register, destination_reg: Register) {
        if let Some(val) = self.memory.read2(source_reg.value) {
            // Update the destination register in the bank
            if let Ok(dest) = self.registers.get_register_mut(destination_reg.id as u8) {
                dest.value = val;
            } else {
                self.halt();
            }
        } else {
            self.halt();
        }
    }

    pub fn write(&mut self, destination_reg: Register, source_reg: Register) {
        if let Err(_) = self.memory.write2(destination_reg.value, source_reg.value) {
            self.halt();
        }
    }

    pub fn copy(&mut self, address_reg: Register, destination_reg: Register) {
        // self.memory.copy(address_reg.value, destination_reg.value, n)
        // destination_reg.value = address_reg.value
    }

    pub fn add(&mut self, address_reg: Register, destination_reg: Register) {
        // destination_reg.value = address_reg.value + destination_reg.value
    }

    /*
        Tick and execute_instruction will load an instruction into the IR and execute it if the machine is not halted.
        It will decode the instruction into the opcode, the register indices and the immediate data and pass this along the instruction.
    */

    pub fn execute_instruction(&self, ir_reg_addr: VMWord) -> Result<()> {
        // Decode the instruction
        let instruction = match self.memory.read2(ir_reg_addr) {
            Some(val) => val,
            None => return Err(VMError::MemoryReadError)
        };

        let opcode = instruction >> 12;
        println!("Opcode: {}", opcode);


  
        // let opcode = instruction >> 12;

        Ok(())
        // TODO: Finish
    }

    // If not halted, execute the instruction
    // It designed to advance the VM by one instruction cycle, loads the next ix address from PC to IR
    // Increments PC to point to next ix
    // Executes the ix currently in the ix register
    // Simulates the fetch-decode-execute cycle typical in CPUs
    // Each VM instance is dedicated to run one program from start to finish.
    pub fn tick(&mut self) -> Result<()> {
        if self.halted {
            return Err(VMError::Halted);
        }

        let mut ir_reg_addr = self
            .registers
            .get_register_read_only(RegisterId::RIR.id())?
            .value;

        let pc_reg_addr = self
            .registers
            .get_register_read_only(RegisterId::RPC.id())?
            .value;

        {
            let ir = self.registers.get_register_mut(RegisterId::RIR.id())?;
            ir.value = pc_reg_addr;
            ir_reg_addr = pc_reg_addr;
        }

        {
            let pc = self.registers.get_register_mut(RegisterId::RPC.id())?;
            pc.value += 1;
        }

        if let Err(error) = self.execute_instruction(ir_reg_addr) {
            self.halt();
            return Err(error);
        }

        Ok(())
    }
}

/*
Instruction set which tells the CPU to do some fundamental task, such as add two numbers. Instructions have opcode (kind of task) and a set of parameters which provide inputs to the task being performed.

Each opcode is one task that the CPU knows how to do.
Each instruction is 16-bit in my case, with the left 4 bits storing the opcode. The rest of the bits are used to store the parameters.

So i decide how much bit/byte to give for my opcode when i decide how much unique operations i want my VM to support
*/
// enum Opcode {
//     NOP,      // No operation
//     ADD,      // Add
//     SUB,      // Subtract
//     MUL,      // Multiply
//     DIV,      // Divide
//     MOV,      // Move value
//     LOAD,     // Load from memory
//     STORE,    // Store to memory
//     JMP,      // Jump
//     BEQ,      // Branch if equal
//     BNE,      // Branch if not equal
//     AND,      // Bitwise AND
//     OR,       // Bitwise OR
//     XOR,      // Bitwise XOR
//     NOT,      // Bitwise NOT
//     HALT      // Stop execution
// }

enum Opcode {
    HALT, // 0x01
    READ,
    WRITE,
    COPY,
    ADD,
}
