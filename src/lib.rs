use crate::{
    bus::BusDevice, memory::LinearMemory, utils::build_simple_program, vm::VM, zk::ZkContext,
};

pub mod bus;
pub mod constants;
pub mod error;
pub mod memory;
pub mod register;
pub mod utils;
pub mod vm;
pub mod zk;
use constants::START_ADDRESS;

pub fn start_vm() {
    println!("VM is running...");

    let program = build_simple_program();
    let mut vm = VM::new();

    // Public inputs, used for the zk logic
    let mut public_inputs = ZkContext::new();
    if let Err(_) = public_inputs.set_public_program(program.clone()) {
        eprintln!("Error settings public inputs for program");
    }

    // This loads (write) the program into memory at the specified addresses (NOT EXECUTE)
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
    vm.enable_trace();

    while !vm.halted {
        if let Err(e) = vm.tick() {
            eprintln!("Vm error: {}", e.message());
            break;
        }
    }

    // Capture the OUTPUT state of the VM
    if let Err(_) = public_inputs.set_public_output(&vm.registers, &vm.memory) {
        eprintln!("Cannot capture the output state from the VM.");
    }

    // println!("Public inputs: {:?}", public_inputs);
    println!("Program: {:?}", public_inputs);

    if let Some(program_result) = vm.memory.read2(START_ADDRESS) {
        println!("The Value at address 0x100 is {}", program_result);
    } else {
        eprintln!("Could not read memory at 0x1000");
    }
}
