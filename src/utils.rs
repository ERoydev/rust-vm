pub fn instruction_builder(opcode: u8, dest: u8, source: u8, immediate: u8) -> u16 {
    // Ensure only 4 bits are used for each field
    // With masking i ensure only the lowest 4 bits of each value are used. So each field fits within its 4-bit slot in the instruction
    let opcode = (opcode & 0xF) as u16;
    let dest = (dest & 0xF) as u16;
    let source = (source & 0xF) as u16;
    let immediate = (immediate & 0xF) as u16;

    // bit: 15 14 13 12 11 10 9 8 7 6 5 4 3 2 1 0
    //       |  |  |  |  |  |  | | | | | | | | | |
    //       -------------------------------------
    //       Most significant           Least significant
    (opcode << 12) | (dest << 8) | (source << 4) | immediate
}

pub fn build_simple_program() -> Vec<u16> {
    let load_imm_ix_rim = instruction_builder(0x05, 0x06, 0x00, 0x05); // Load_imm 0x05 into RIM

    let copy_ix_r0 = instruction_builder(0x01, 0x00, 0x06, 0x00); // Copy 0x05 from RIM to R0

    let load_imm_ix_rim2 = instruction_builder(0x05, 0x06, 0x00, 0x03); // Load_imm 0x03 into RIM

    let copy_ix_r1 = instruction_builder(0x01, 0x01, 0x06, 0x00); // Copy 0x03 from RIM to R1

    // Step ADD r0 and r1
    let add_ix = instruction_builder(0x04, 0x00, 0x01, 0x00);

    // Store the result from r0 into memory
    let store_out = instruction_builder(0x06, 0x00, 0x00, 0x00);

    // Program
    vec![
        load_imm_ix_rim,
        copy_ix_r0,
        load_imm_ix_rim2,
        copy_ix_r1,
        add_ix,
        store_out,
    ]
}
