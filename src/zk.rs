use crate::{
    bus::BusDevice,
    constants::{BN254_MODULUS, VMWord},
    error::Result,
    register::RegisterBank,
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
    // The bellow hashes are computed in the following order -> Sha256(data) -> Poseidon::hash(sha256_hashed_data)
    pub public_program_hash: Fr,
    pub public_input_hash: Fr,
    pub public_output_hash: Fr, // concat(final_registers, final_memory)

    // Private witness,
    pub raw_program_hash_sha: Fr,
}

impl ZkContext {
    pub fn new() -> Self {
        Self {
            public_program_hash: Fr::ZERO,
            public_input_hash: Fr::ZERO,
            public_output_hash: Fr::ZERO,
            raw_program_hash_sha: Fr::ZERO,
        }
    }

    pub fn set_public_program(&mut self, program: Vec<VMWord>) -> Result<()> {
        let serialized_program = serialize(&program).unwrap();
        let sha256_hashed = Sha256Hash::hash(&serialized_program);
        // Save the hash as a private representation of raw_program witness
        let sha_to_bn254_field = Sha256Hash::_sha256_to_field(&sha256_hashed);
        self.raw_program_hash_sha = sha_to_bn254_field;

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
        memory: &Box<dyn BusDevice>,
    ) -> Result<()> {
        // Serialize registers and memory
        let reg_bytes = wincode::serialize(registers).unwrap();
        let mem_bytes_vec = memory.as_bytes();
        let sha_combined_hash = Sha256Hash::hash_multiple(&[&reg_bytes, &mem_bytes_vec]);
        let sha_to_bn254_field = Sha256Hash::_sha256_to_field(&sha_combined_hash);
        let poseidon_hash = ZkContext::_compute_poseidon_hash(sha_to_bn254_field).unwrap();
        self.public_output_hash = poseidon_hash;
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
    pub fn hash(bytes: &Vec<u8>) -> BigUint {
        let mut hasher = Sha256::new();
        hasher.update(bytes);

        let hashed_value = hasher.finalize();
        let hashed_big_num = BigUint::from_bytes_be(&hashed_value);
        hashed_big_num
    }

    // Combine multiple data into one hash
    pub fn hash_multiple(data: &[&[u8]]) -> BigUint {
        let mut hasher = Sha256::new();
        for slice in data {
            hasher.update(slice);
        }
        let hashed_value = hasher.finalize();
        let hashed_big_num = BigUint::from_bytes_be(&hashed_value);
        hashed_big_num
    }

    pub fn _sha256_to_field(sha256: &BigUint) -> Fr {
        /*
            Finite fields of BN254 have a prime modulus close to a 254-bit value
            Sha256 produces max a 256-bit value possibly exceeding the Finite Field, Poseidon fails with `InputLargerThanModulus`
            Solution: reduce Sha256 output modulo to make it <= 254 bits, so poseidon can accept it
        */
        let modulus = BigUint::from(BN254_MODULUS);
        let reduced_sha = sha256 % modulus;

        let bytes = reduced_sha.to_bytes_be();
        let field = Fr::from_be_bytes_mod_order(&bytes);
        field
    }
}
