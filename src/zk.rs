use crate::{
    bus::BusDevice,
    constants::{BN254_MODULUS, START_ADDRESS, VMWord},
    error::{Result, VMError},
    register::{RegisterBank, RegisterId},
};
use ark_bn254::Fr;
use ark_ff::{AdditiveGroup, PrimeField};
use light_poseidon::{Poseidon, PoseidonHasher};
use num_bigint::BigUint;
use sha2::{Digest, Sha256};
use wincode;
use wincode::serialize;

#[derive(Debug)]
pub struct ZkContext {
    // Every Public input must be a hash performed using poseidon -> Sha256(data) -> Poseidon::hash(sha256_hashed_data)
    pub public_program_hash: Fr,
    pub public_output_hash: Fr, // concat(final_registers, final_memory)

    // Private witness -> Every private witness must be a hashed Field using Sha256 % BN254_MODULUS
    pub private_program_sha254: Fr,
    pub private_output_sha254: Fr,
}

impl Default for ZkContext {
    fn default() -> Self {
        Self {
            public_program_hash: Fr::ZERO,
            public_output_hash: Fr::ZERO,
            private_program_sha254: Fr::ZERO,
            private_output_sha254: Fr::ZERO,
        }
    }
}

impl ZkContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_public_program(&mut self, program: Vec<VMWord>) -> Result<()> {
        let serialized_program = serialize(&program).unwrap();
        let sha_to_bn254_field = Sha256Hash::hash(&serialized_program);
        // Save the hash as a private representation of raw_program witness
        self.private_program_sha254 = sha_to_bn254_field;

        // Hash the public program using poseidon
        let poseidon_hashed = ZkContext::_compute_poseidon_hash(sha_to_bn254_field).unwrap();
        self.public_program_hash = poseidon_hashed;
        Ok(())
    }

    pub fn set_public_input(&mut self) {
        // Currently programs executed by this vm doesn't support inputs
        todo!()
    }

    pub fn set_public_output(
        &mut self,
        registers: &RegisterBank,
        memory: &dyn BusDevice,
    ) -> Result<()> {
        // Serialize registers and memory
        let pc = registers
            .get_register_read_only(RegisterId::RPC.id())?
            .value as usize;
        let output_from_r0 = memory
            .read2(START_ADDRESS)
            .ok_or(VMError::MemoryReadError)?;

        let output_state = serialize(&output_from_r0).unwrap();
        let final_memory_subset = memory.get_subset_of_memory(START_ADDRESS as usize, pc);
        let final_registers_state = wincode::serialize(registers).unwrap();

        let sha_to_bn254_field = Sha256Hash::hash_multiple(&[
            &output_state,
            &final_memory_subset,
            &final_registers_state,
        ]);

        let poseidon_hash = ZkContext::_compute_poseidon_hash(sha_to_bn254_field).unwrap();
        self.public_output_hash = poseidon_hash;
        self.private_output_sha254 = sha_to_bn254_field;
        Ok(())
    }

    pub fn _compute_poseidon_hash(sha_hashed: Fr) -> Result<Fr> {
        let mut poseidon = Poseidon::<Fr>::new_circom(1).unwrap();
        let hash = poseidon.hash(&[sha_hashed]).unwrap();
        Ok(hash)
    }
}

pub struct Sha256Hash {}

impl Sha256Hash {
    /// Hashes the input bytes using SHA256, reduces the result modulo the BN254 field,
    /// and returns the result as a BN254 field element (Fr).
    /// This ensures the hash fits within the field for use in ZK circuits.
    pub fn hash(bytes: &Vec<u8>) -> Fr {
        let mut hasher = Sha256::new();
        hasher.update(bytes);

        let hashed_value = hasher.finalize();
        let hashed_big_num = BigUint::from_bytes_be(&hashed_value);
        Sha256Hash::__sha256_to_field(&hashed_big_num)
    }

    /// Hashes multiple byte slices using SHA256, concatenates them, reduces the result modulo the BN254 field,
    /// and returns the result as a BN254 field element (Fr).
    /// This is useful for hashing combined data (e.g., registers and memory) into a single field element for ZK circuits.
    pub fn hash_multiple(data: &[&[u8]]) -> Fr {
        let mut hasher = Sha256::new();
        for slice in data {
            hasher.update(slice);
        }
        let hashed_value = hasher.finalize();
        let hashed_big_num = BigUint::from_bytes_be(&hashed_value);
        Sha256Hash::__sha256_to_field(&hashed_big_num)
    }

    fn __sha256_to_field(sha256: &BigUint) -> Fr {
        /*
            Finite fields of BN254 have a prime modulus close to a 254-bit value
            Sha256 produces max a 256-bit value possibly exceeding the Finite Field, Poseidon fails with `InputLargerThanModulus`
            Solution: reduce Sha256 output modulo to make it <= 254 bits, so poseidon can accept it
        */
        let modulus = BigUint::from(BN254_MODULUS);
        let reduced_sha = sha256 % modulus;

        let bytes = reduced_sha.to_bytes_be();
        Fr::from_be_bytes_mod_order(&bytes)
    }
}
