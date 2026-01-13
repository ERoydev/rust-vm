use sha2::{Digest, Sha256};
use crate::{bus::BusDevice, constants::VMWord, register::RegisterBank, error::{Result}};
use wincode;


#[derive(Debug)]
pub struct PublicInputs {
    pub program_hash: [u8; 32], 
    pub input_hash: [u8; 32],
    pub output_hash: [u8; 32], // concat(final_registers, final_memory)
}

impl PublicInputs {
    pub fn new() -> Self {
        Self {
            program_hash: [0u8; 32],
            input_hash: [0u8; 32],
            output_hash: [0u8; 32],
        }
    }

    pub fn set_program(&mut self, program: Vec<VMWord>) {
        let mut hasher = Sha256::new();
        // Convert &[u16] to &[u8] safely
        let bytes = unsafe {
            std::slice::from_raw_parts(program.as_ptr() as *const u8, program.len() * 2)
        };
        hasher.update(bytes);
        let hashed_program = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&hashed_program);
        self.program_hash = arr;
    }

    pub fn set_input(&mut self) {
        // Currently programs executed by this vm doesn't support inputs
        todo!()
    }

    pub fn set_output(&mut self, registers: &RegisterBank, memory: &Box<dyn BusDevice>) -> Result<()> {
        // TODO: Implement error handling
        let mut hasher = Sha256::new();
        
        // Serialize registers
        println!("Registers: {:?}", registers);
        let reg_bytes = wincode::serialize(registers).unwrap(); // TODO: Hash of the registers is not deterministic
        let mem_bytes_vec = memory.as_bytes();
        
        // println!("\nReg Bytes: {:?}", reg_bytes);
        // println!("\nMem Bytes: {:?}", mem_bytes_vec);
        // hasher.update(&reg_bytes);
        // hasher.update(mem_bytes_vec);
        // let output_hash = hasher.finalize();

        // println!("Output Hash: {:?}", output_hash);

        Ok(())
    }


}