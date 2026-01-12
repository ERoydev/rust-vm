use crate::{
    bus::BusDevice, memory::LinearMemory, register::{Register, RegisterId}, utils::{build_simple_program, instruction_builder}, vm::VM
};

pub mod bus;
pub mod error;
pub mod memory;
pub mod register;
pub mod utils;
pub mod vm;

pub fn start_vm() {
    println!("VM is running...");
    
    let program = build_simple_program();
    let mut vm = VM::new();
    println!("Raw Program to execute: {:?}", program);

    // This loads (write) the program into memory at the specified addresses (NOT EXECUTE)
    let mut memory = LinearMemory::new(5000);
    for (i, add_reg) in program.iter().enumerate() {
        const START_ADDRESS: u16 = 0x100; // I use this as start address, so i will first 256 bytes reserved for Program Segment Prefix
        let address_to_write = u16::try_from(i)
            .expect("Value out of range for u16")
            .checked_add(START_ADDRESS)
            .expect("Index + 0x100 is out of range");

        if let Err(e) = memory.write2(address_to_write, *add_reg) {
            println!("Writing on memory error on location: {}, err: {}", address_to_write, e);
        }
    }

    vm.set_memory(Box::new(memory));

    let _ = vm.tick();
    // while !vm.halted {
    //     if let Err(e) = vm.tick() {
    //         eprintln!("Vm error: {}", e.message());
    //         break;
    //     }
    // }

    if let Some(program_result) = vm.memory.read(0x100) {
        println!("The value at address 0x100 is {}", program_result);
    } else {
        eprintln!("Could not read memory at 0x1000");
    }
}
