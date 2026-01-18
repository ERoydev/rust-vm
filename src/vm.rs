#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;

use ark_bn254::Fr;
use ark_ff::AdditiveGroup;
use wincode::serialize;

use crate::constants::{START_ADDRESS, VMWord};
use crate::error::Result;
use crate::zk::{Sha256Hash, ZkContext};
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
#[derive(Debug)]
pub struct VM {
    pub registers: RegisterBank,
    pub memory: Box<dyn BusDevice>, // main memory
    pub halted: bool, // Signal when the VM should stop processing instructions, after program finishes or encounter a fatal error

    pub trace_enabled: bool,
    pub trace_buffer: Vec<TraceEntry>, // store trace entries
    pub zk_output_enabled: bool,
}

impl Default for VM {
    fn default() -> Self {
        Self {
            registers: RegisterBank::new(),
            memory: Box::new(LinearMemory::new(0)),
            halted: false,
            trace_enabled: false,
            trace_buffer: Vec::new(),
            zk_output_enabled: false,
        }
    }
}

impl VM {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_memory(&mut self, memory: Box<dyn BusDevice>) {
        self.memory = memory;
        println!("Set a new memory");
    }

    pub fn enable_trace(&mut self) {
        self.trace_enabled = true;
        println!("Trace enabled");
    }

    pub fn enable_zk_output(&mut self) {
        self.zk_output_enabled = true;
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
        let reg = if reg_i == RegisterId::RIM.id() && imm_value != 0 {
            let tmp = self.registers.get_register_mut(reg_i)?;
            tmp.value = imm_value;
            *tmp
        } else {
            // When i deref a mut ref i return a copy of the Register, not a ref to the original
            *self.registers.get_register_mut(reg_i)?
        };
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

    pub fn _write_logs<T: std::fmt::Debug>(data: T, file_name: &str) {
        let log_dir = ".logs";
        // Create the directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(log_dir) {
            eprintln!("Failed to create log directory: {}", e);
            return;
        }

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(format!("{}/{}.log", log_dir, file_name))
        {
            writeln!(file, "{:#?}", data).unwrap();
        }
    }

    fn _parse_private_inputs(&self) {
        // Combines pc, mem_at_pc_loc, register at that step, opcode at that step into Poseidon hash
        let mut pub_program_state: Vec<Fr> = vec![];
        let mut private_program_state: Vec<Fr> = vec![];

        for entry in &self.trace_buffer {
            let mut reg_array = [0u16; 7];

            for (idx, reg) in entry.registers.iter() {
                reg_array[*idx as usize] = reg.value;
            }

            let memory_at_location = self.memory.get_specific_memory_location(entry.pc as usize);
            let mem_bytes = serialize(&memory_at_location).unwrap();
            let register_bytes: Vec<u8> = serialize(&reg_array).unwrap();
            let pc_bytes = serialize(&entry.pc).unwrap();
            let opcode_bytes = serialize(&(entry.opcode as u16)).unwrap();

            let hashed_state =
                Sha256Hash::hash_multiple(&[&mem_bytes, &register_bytes, &pc_bytes, &opcode_bytes]);
            let poseidon_hash = ZkContext::_compute_poseidon_hash(hashed_state).unwrap();

            pub_program_state.push(poseidon_hash);
            private_program_state.push(hashed_state);
        }

        if let Ok(state) = std::env::var("ZK_STATE_CAPACITY") {
            // Add dummy states to fit zk program expected state capacity
            let current_state_len = pub_program_state.len();
            let state_capacity = state.parse::<usize>().unwrap() - current_state_len;
            VM::_write_logs(current_state_len, "state_len");

            for _i in 0..state_capacity {
                pub_program_state.push(Fr::ZERO);
                private_program_state.push(Fr::ZERO);
            }

            VM::_write_logs(pub_program_state, "public_program_state");
            VM::_write_logs(private_program_state, "private_program_state");
        } else {
            eprintln!("ZK_STATE_CAPACITY is not defined in .env file!");
        }
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
        if self.zk_output_enabled {
            self._parse_private_inputs();
        }

        self.halted = true;
    }

    fn write(&mut self, source_reg: Register, destination_reg: Register) {
        // dst_reg is address
        if self
            .memory
            .write2(destination_reg.value, source_reg.value)
            .is_err()
        {
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
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::BusDevice;
    use crate::constants::VmAddr;
    use crate::error::VMError;
    use crate::register::RegisterId;
    use crate::utils::build_simple_program;

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
            let low_byte = self.memory[idx] as u16;
            let high_byte = self.memory[idx + 1] as u16;
            (high_byte << 8) | low_byte
        }

        fn get_subset_of_memory(&self, start_addr: usize, end_addr: usize) -> Vec<u8> {
            self.memory[start_addr..end_addr].to_vec()
        }
    }

    #[test]
    fn test_vm_initialization() {
        let vm = VM::new();
        assert_eq!(vm.halted, false);
        assert_eq!(vm.trace_enabled, false);
        assert_eq!(vm.trace_buffer.len(), 0);
    }

    #[test]
    fn test_set_memory() {
        let mut vm = VM::new();
        let dummy = Box::new(MockBus::new());
        vm.set_memory(dummy);
        assert_eq!(vm.memory.memory_range(), 1024);
    }

    #[test]
    fn test_enable_trace() {
        let mut vm = VM::new();
        vm.enable_trace();
        assert!(vm.trace_enabled);
    }

    #[test]
    fn test_tick_halted() {
        let mut vm = VM::new();
        vm.halted = true;
        let result = vm.tick();
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_instruction_dispatch_with_halt() {
        let mut vm = VM::new();
        let dummy = Box::new(MockBus::new());
        vm.set_memory(dummy);
        // Write a HALT instruction at address 0
        let halt_opcode: u16 = 0 << 12;
        vm.memory.write2(0, halt_opcode).unwrap();
        // Set PC to 0
        let rpc = vm.registers.get_register_mut(RegisterId::RPC.id()).unwrap();
        rpc.value = 0;
        let result = vm.tick();
        assert!(vm.halted);
        assert!(result.is_ok());
    }

    #[test]
    fn text_execute_instruction_registers_and_pc() {
        let program = build_simple_program();
        let mut vm = VM::new();

        let mut memory = LinearMemory::new(5000);
        for (i, add_reg) in program.iter().enumerate() {
            let address_to_write = u16::try_from(i)
                // START_ADDRESS + (i as u16) * 2;
                .expect("Value out of range for u16")
                .checked_mul(2) // Implementation of a for loop step by 2
                .expect("i * 2 failed")
                .checked_add(START_ADDRESS)
                .expect("Index + 0x100 out of range");

            println!("\nAddress: {}, Value: {}", address_to_write, add_reg);

            if let Err(e) = memory.write2(address_to_write, *add_reg) {
                println!(
                    "Writing on memory error on location: {}, err: {}",
                    address_to_write, e
                );
            }
        }

        vm.set_memory(Box::new(memory));
        let mut step = 0;
        let expected_pcs: Vec<u16> = vec![258, 260, 262, 264, 266, 268, 270];
        let expected_registers = vec![
            // Step 0
            [0, 0, 0, 0, 258, 22021, 5],
            // Step 1
            [5, 0, 0, 0, 260, 4192, 5],
            // Step 2
            [5, 0, 0, 0, 262, 22019, 3],
            // Step 3
            [5, 3, 0, 0, 264, 4448, 3],
            // Step 4
            [8, 3, 0, 0, 266, 16400, 3],
            [8, 3, 0, 0, 268, 24576, 3],
            [8, 3, 0, 0, 270, 0, 3],
        ];
        let expected_mem = vec![4192, 22019, 4448, 16400, 24576, 0, 0];

        while !vm.halted {
            if let Err(e) = vm.tick() {
                eprintln!("Vm error: {}", e.message());
                break;
            } else {
                // Test rpc step
                let rpc = vm.registers.get_register_mut(RegisterId::RPC.id()).unwrap();
                assert_eq!(rpc.value, expected_pcs[step]);

                // test memory at location
                let mem = vm.memory.get_specific_memory_location(rpc.value as usize);
                assert_eq!(mem, expected_mem[step]);

                // Test register value at each step
                let reg_map = &vm.registers.register_map;
                let actual: Vec<u16> = (0..7).map(|i| reg_map[&i].value).collect();
                assert_eq!(actual, expected_registers[step]);

                step += 1;
            }
        }
    }
}
