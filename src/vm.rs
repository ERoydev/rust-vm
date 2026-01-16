#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;

use crate::constants::{START_ADDRESS, VMWord};
use crate::error::Result;
use crate::{
    bus::BusDevice,
    error::VMError,
    memory::LinearMemory,
    register::{Register, RegisterBank, RegisterId},
};

// The VM config
pub struct Config {}

#[derive(Debug, Clone)]
pub struct TraceEntry {
    pc: VMWord,

    opcode: Opcode,
    dst: u8,
    src: u8,
    imm: VMWord,

    registers: BTreeMap<u8, Register>, // TODO: Storing registers like that is not the most efficient way, but i am going to leave it for now, to experiment with zk first.
}

impl TraceEntry {
    fn new(
        pc: VMWord,
        opcode: Opcode,
        dst: u8,
        src: u8,
        imm: VMWord,
        registers: BTreeMap<u8, Register>,
    ) -> Self {
        Self {
            pc,
            opcode,
            dst,
            src,
            imm,
            registers,
        }
    }
}

pub trait VMOperations {
    fn halt(&mut self, _: Register, _: Register);
    fn write(&mut self, source_reg: Register, destination_reg: Register);
    fn copy(&mut self, source_reg: Register, destination_reg: Register);
    fn add(&mut self, source_reg: Register, destination_reg: Register);
    fn load(&mut self, source_reg: Register, destination_reg: Register);
    fn load_imm(&mut self, _: Register, _: Register);
    fn store_out(&mut self, source_reg: Register, _: Register);
}

// It will simulate the computer for the 16bit VM
pub struct VM {
    pub registers: RegisterBank,
    pub memory: Box<dyn BusDevice>, // main memory
    pub halted: bool, // Signal when the VM should stop processing instructions, after program finishes or encounter a fatal error

    pub trace_enabled: bool,
    pub trace_buffer: Vec<TraceEntry>, // store trace entries
}

impl VM {
    pub fn new() -> Self {
        Self {
            registers: RegisterBank::new(),
            memory: Box::new(LinearMemory::new(0)),
            halted: false,
            trace_enabled: false,
            trace_buffer: Vec::new(),
        }
    }

    pub fn set_memory(&mut self, memory: Box<dyn BusDevice>) {
        self.memory = memory;
        println!("Set a new memory");
    }

    pub fn enable_trace(&mut self) {
        self.trace_enabled = true;
        println!("Trace enabled");
    }

    /*
        Tick and execute_instruction will load an instruction into the IR and execute it if the machine is not halted.
        It will decode the instruction into the opcode, the register indices and the immediate data and pass this along the instruction.
    */
    pub fn execute_instruction(&mut self, instruction: VMWord) -> Result<()> {
        // Decode the instruction
        let opcode = Opcode::try_from((instruction >> 12) as u8)?;
        let dest_reg_i = ((instruction & 0x0F00) >> 8) as u8;
        let source_reg_i = ((instruction & 0x00F0) >> 4) as u8;
        let immediate_value = instruction & 0x000F;

        if self.trace_enabled {
            self.trace(opcode, dest_reg_i, source_reg_i, immediate_value);
        }

        let dest_reg = self.resolve_register_or_immediate(dest_reg_i, immediate_value)?;
        let src_reg = self.resolve_register_or_immediate(source_reg_i, immediate_value)?;

        // Opcode dispatcher invokes the VM to work with the register operations
        match opcode {
            Opcode::HALT => self.halt(src_reg, dest_reg),
            Opcode::WRITE => self.write(src_reg, dest_reg),
            Opcode::COPY => self.copy(src_reg, dest_reg),
            Opcode::ADD => self.add(src_reg, dest_reg),
            Opcode::LOAD => self.load(src_reg, dest_reg),
            Opcode::LOAD_IMM => self.load_imm(src_reg, dest_reg),
            Opcode::STORE_OUT => self.store_out(src_reg, dest_reg),
        }

        Ok(())
    }

    // If not halted, execute the instruction
    // It designed to advance the VM by one instruction cycle, loads the next ix address from PC to IR
    // Increments PC to point to next ix
    // Executes the instruction currently in the instruction register
    // Simulates the fetch-decode-execute cycle typical in CPUs
    // Each VM instance is dedicated to run one program from start to finish.
    pub fn tick(&mut self) -> Result<()> {
        if self.halted {
            return Err(VMError::Halted);
        }

        // This holds the start address to read from memory
        let pc_reg_addr = self
            .registers
            .get_register_read_only(RegisterId::RPC.id())?
            .value;

        // TODO: Improve error handling
        let raw_instruction: u16 = self.memory.read2(pc_reg_addr).unwrap();

        {
            let ir = self.registers.get_register_mut(RegisterId::RIR.id())?;
            ir.value = raw_instruction;
        }

        {
            let pc = self.registers.get_register_mut(RegisterId::RPC.id())?;
            pc.inc_program_counter()?;
        }

        if let Err(error) = self.execute_instruction(raw_instruction) {
            self.halted = true;
            return Err(error);
        }

        Ok(())
    }

    // If reg is RIM it will load the immediate value into that register immediately
    fn resolve_register_or_immediate(&mut self, reg_i: u8, imm_value: u16) -> Result<Register> {
        let reg;
        if reg_i == RegisterId::RIM.id() && imm_value != 0 {
            let tmp = self.registers.get_register_mut(reg_i)?;
            tmp.value = imm_value;
            reg = *tmp
        } else {
            // When i deref a mut ref i return a copy of the Register, not a ref to the original
            reg = *self.registers.get_register_mut(reg_i)?;
        }
        Ok(reg)
    }

    fn trace(&mut self, opcode: Opcode, dst: u8, src: u8, imm: VMWord) {
        // TODO: Improve error handling
        let pc_addr = self
            .registers
            .get_register_read_only(RegisterId::RPC.id())
            .unwrap()
            .value;
        self.trace_buffer.push(TraceEntry::new(
            pc_addr,
            opcode,
            dst,
            src,
            imm,
            self.registers.register_map.clone(),
        ));
    }

    fn _write_logs<T: std::fmt::Debug>(data: T, file_name: &str) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(format!(".logs/{file_name}.log"))
        {
            writeln!(file, "{:#?}", data).unwrap();
        }
    }

    fn _parse_private_inputs(&self) {
        let mut registers = vec![];
        let mut pc = vec![];
        let mut opcode = vec![];
        let mut reg_pairs = vec![];

        for entry in &self.trace_buffer {
            let mut reg_array = [0u16; 7];
            let mut reg_pair_nested_array = [0usize; 3];

            for (idx, reg) in entry.registers.iter() {
                reg_array[*idx as usize] = reg.value;
            }

            [
                reg_pair_nested_array[0],
                reg_pair_nested_array[1],
                reg_pair_nested_array[2],
            ] = [entry.dst as usize, entry.src as usize, entry.imm as usize];
            reg_pairs.push(reg_pair_nested_array);

            registers.push(reg_array);
            pc.push(entry.pc);
            opcode.push(entry.opcode as u16);
        }

        let last_memory_addr = *pc.last().unwrap() as usize;
        let memory_vec = self.memory.get_subset_of_memory(0x100, last_memory_addr);

        VM::_write_logs(memory_vec, "memory_subset");
        VM::_write_logs(registers, "registers");
        VM::_write_logs(pc, "pc");
        VM::_write_logs(opcode, "opcode");
        VM::_write_logs(reg_pairs, "reg_pairs");
    }
}

/// Implements the core instruction set operations for the VM.
///
/// These methods correspond to the fundamental instructions that the VM can execute,
/// such as halting, reading, writing, copying, and adding values.
/// Each method is invoked in response to a specific opcode during program execution.
impl VMOperations for VM {
    // TODO: Improve error handling for VMOperations
    fn halt(&mut self, _: Register, _: Register) {
        VM::_write_logs(&self.trace_buffer, "vm_trace");
        self._parse_private_inputs();

        self.halted = true;
    }

    fn write(&mut self, source_reg: Register, destination_reg: Register) {
        // dst_reg is address
        if let Err(_) = self.memory.write2(destination_reg.value, source_reg.value) {
            self.halted = true;
        }
    }

    fn copy(&mut self, source_reg: Register, destination_reg: Register) {
        let dest_register = self
            .registers
            .get_register_mut(destination_reg.id.id())
            .unwrap();
        dest_register.value = source_reg.value;
    }

    fn add(&mut self, source_reg: Register, destination_reg: Register) {
        let result = source_reg
            .value
            .checked_add(destination_reg.value)
            .expect("Add instruction failed with overflow");
        let dest_register = self
            .registers
            .get_register_mut(destination_reg.id.id())
            .unwrap();
        dest_register.value = result;
    }

    fn load(&mut self, source_reg: Register, destination_reg: Register) {
        if let Some(val) = self.memory.read2(source_reg.value) {
            // When load reg.value is interpret as an address to a memory location
            let dest_register = self
                .registers
                .get_register_mut(destination_reg.id.id())
                .unwrap();
            dest_register.value = val;
        } else {
            eprintln!("LOAD instruction fails");
            self.halted = true;
        }
    }

    fn load_imm(&mut self, _: Register, _: Register) {
        // print!("Immediate value loaded successfully");
    }

    fn store_out(&mut self, source_reg: Register, _: Register) {
        if let Err(err) = self.memory.write2(START_ADDRESS, source_reg.value) {
            eprintln!("Store out error: {}", err.message());
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

/// It depends on the OPCODE, sometimes reg.value is a bytes holding data already taken from memory, at other opcodes reg.value is an address pointing to a location in memory
#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
enum Opcode {
    // These are so called mnemonics, human-readable representations of machine instructions, used to make VM ISA easier to understand
    HALT,
    COPY,      // register <- register
    LOAD,      // register <- memory[address in register]
    WRITE,     // memory[address in register] <- register
    ADD,       // register <- register + register
    LOAD_IMM,  // register <- immediage
    STORE_OUT, // store result from R0 to memory at start address
}

impl Opcode {
    pub fn id(&self) -> u8 {
        *self as u8
    }
}

impl TryFrom<u8> for Opcode {
    type Error = VMError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Opcode::HALT),
            1 => Ok(Opcode::COPY),
            2 => Ok(Opcode::LOAD),
            3 => Ok(Opcode::WRITE),
            4 => Ok(Opcode::ADD),
            5 => Ok(Opcode::LOAD_IMM),
            6 => Ok(Opcode::STORE_OUT),

            _ => Err(VMError::OpcodeDoesNotExist),
        }
    }
}
