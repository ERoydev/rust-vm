use crate::{
    bus::BusDevice, memory::LinearMemory, register::{Register, RegisterId}, utils::{build_simple_program, instruction_builder}, vm::VM
};

pub mod bus;
pub mod error;
pub mod memory;
pub mod register;
pub mod utils;
pub mod vm;

fn main() {
    println!("VM is running...");
    
    let program = build_simple_program();
    let vm = VM::new();

    // This loads (write) the program into memory at the specified addresses (NOT EXECUTE)
    for (i, add_reg) in program.iter().enumerate() {
        const START_ADDRESS: u16 = 0x100; // I use this as start address, so i will first 256 bytes reserved for Program Segment Prefix
        let address_to_write = u16::try_from(i)
            .expect("Value out of range for u16")
            .checked_add(START_ADDRESS)
            .expect("Index + 0x100 is out of range");

        let mut mem = LinearMemory::new(5000);
        if let Err(e) = mem.write2(address_to_write, *add_reg) {
            println!("Writing on memory error on location: {}, err: {}", address_to_write, e);
        }
    }

    while !vm.halted {
        if let Err(e) = vm.tick() {
            eprintln!("Vm error: {}", e.message());
            break;
        }
    }

    if let Some(program_result) = vm.memory.read(0x100) {
        println!("The value at address 0x100 is {}", program_result);
    } else {
        eprintln!("Could not read memory at 0x1000");
    }
}
