#![allow(dead_code)]

use crate::constants::VmAddr;
use crate::error::Result;
use crate::{
    bus::BusDevice,
    error::VMError,
    memory::LinearMemory,
    register::{Register, RegisterBank, RegisterId},
};

// The VM config
pub struct Config {}

pub trait VMOperations {
    fn halt(&mut self, _: Register, _: Register);
    fn read(&mut self, source_reg: Register, destination_reg: Register);
    fn write(&mut self, source_reg: Register, destination_reg: Register);
    fn copy(&mut self, source_reg: Register, destination_reg: Register);
    fn add(&mut self, source_reg: Register, destination_reg: Register);
}

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
            Err(_) => Err(VMError::UnknownRegister),
        }
    }

    /*
        Tick and execute_instruction will load an instruction into the IR and execute it if the machine is not halted.
        It will decode the instruction into the opcode, the register indices and the immediate data and pass this along the instruction.
    */

    pub fn execute_instruction(&mut self, ir_reg_addr: VmAddr) -> Result<()> {
        // Decode the instruction
        let instruction = match self.memory.read2(ir_reg_addr) {
            Some(val) => val,
            None => return Err(VMError::MemoryReadError),
        };

        let opcode = Opcode::try_from((instruction >> 12) as u8)?;
        println!("Address called: {}", ir_reg_addr);
        println!("Instruction: {:016b}", instruction);
        println!("OPCODE RECEIVED: {:?}", opcode);
        let dest_reg_i = ((instruction & 0x0F00) >> 8) as u8;
        let source_reg_i = ((instruction & 0x00F0) >> 4) as u8;
        let immediate_value = instruction & 0x000F;

        let dest_reg = self.resolve_register_or_immediate(dest_reg_i, immediate_value)?;
        let src_reg = self.resolve_register_or_immediate(source_reg_i, immediate_value)?;

        // Opcode dispatcher invokes the VM to work with the register operations
        match opcode {
            Opcode::HALT => self.halt(src_reg, dest_reg),
            Opcode::READ => self.read(src_reg, dest_reg),
            Opcode::WRITE => self.write(src_reg, dest_reg),
            Opcode::COPY => self.copy(src_reg, dest_reg),
            Opcode::ADD => self.add(src_reg, dest_reg),
        }

        Ok(())
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
            // TODO: The error is here since in memory i store it in u8 bytes, while i have 16-bit VM which means that i should read/write 2 bytes at a time
            pc.value += 1;
        }

        if let Err(error) = self.execute_instruction(ir_reg_addr) {
            self.halted = true;
            return Err(error);
        }

        Ok(())
    }

    fn resolve_register_or_immediate(&mut self, reg_i: u8, imm_value: u16) -> Result<Register> {
        let reg;
        if reg_i == RegisterId::RIM.id() {
            reg = Register::new(RegisterId::RIM, imm_value);
        } else {
            reg = *self.registers.get_register_mut(reg_i)?;
        }
        Ok(reg)
    }
}

/// Implements the core instruction set operations for the VM.
///
/// These methods correspond to the fundamental instructions that the VM can execute,
/// such as halting, reading, writing, copying, and adding values.
/// Each method is invoked in response to a specific opcode during program execution.
impl VMOperations for VM {
    fn halt(&mut self, _: Register, _: Register) {
        self.halted = true;
    }

    fn read(&mut self, source_reg: Register, destination_reg: Register) {
        if let Some(val) = self.memory.read2(source_reg.value) {
            // Update the destination register in the bank
            if let Ok(dest) = self.registers.get_register_mut(destination_reg.id as u8) {
                dest.value = val;
            } else {
                self.halted = true;
            }
        } else {
            self.halted = true;
        }
    }

    fn write(&mut self, source_reg: Register, destination_reg: Register) {
        if let Err(_) = self.memory.write2(destination_reg.value, source_reg.value) {
            self.halted = true;
        }
    }

    fn copy(&mut self, source_reg: Register, destination_reg: Register) {
        if let Err(error) = self.memory.copy(source_reg.value, destination_reg.value) {
            eprintln!("COPY error: {}", error.message());
            self.halted = true;
        }
    }

    fn add(&mut self, source_reg: Register, destination_reg: Register) {
        println!("Add triggered");

        if let Err(error) = self.memory.add(source_reg.value, destination_reg.value) {
            eprintln!("ADD error: {}", error.message());
            self.halted = true;
        }
    }
}

/*
Instruction set which tells the CPU to do some fundamental task, such as add two numbers. Instructions have opcode (kind of task) and a set of parameters which provide inputs to the task being performed.

Each opcode is one task that the CPU knows how to do.
Each instruction is 16-bit in my case, with the left 4 bits storing the opcode. The rest of the bits are used to store the parameters.

So i decide how much bit/byte to give for my opcode when i decide how much unique operations i want my VM to support
*/
// enum Opcode {
//     HALT      // Stop execution
//     READ,
//     WRITE,
//     COPY,
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
//     NOP,      // No operation
// }

#[derive(Debug)]
enum Opcode {
    HALT, // 0x00
    READ, // 0x01
    WRITE,
    COPY,
    ADD,
}

impl TryFrom<u8> for Opcode {
    type Error = VMError;

    fn try_from(value: u8) -> Result<Self> {
        println!("Opcode value: {}", value);
        match value {
            0 => Ok(Opcode::HALT),
            1 => Ok(Opcode::READ),
            2 => Ok(Opcode::WRITE),
            3 => Ok(Opcode::COPY),
            4 => Ok(Opcode::ADD),
            _ => Err(VMError::OpcodeDoesNotExist),
        }
    }
}
