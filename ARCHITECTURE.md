

## Anatomy of an instruction

1. For a 16-bit VM my instruction will be like that:
    4 bits: opcode (what operation to perform)
    4 bits: register 1 (often the destination register)
    4 bits: register 2 (often the source register)
    4 bits: immediate value (a small constant, or sometimes used for other purposes) + used for offset at the same time (instruction decoder will decide how to interpret those bits based on the opcode)

2. Clarification  
- register 1 -> is the output or destination where the result goes
- register 2 -> is the input or source what to operate on


3. I can improve in future by adding `offset`, i need it for instruction that access memory(load/store), performs jumps or branches. Arithmetic instructions just operate on register value or immediates, so currently no offset is required.