# 16-bit Virtual Machine in Rust

[![Tests][test-badge]][test-workflow-url]

[test-badge]: https://github.com/LimeChain/codama-dart/actions/workflows/main.yml/badge.svg
[test-workflow-url]: https://github.com/LimeChain/codama-dart/actions/workflows/main.yml

This project is a simple, educational 16-bit virtual machine (VM) written in Rust. It is designed to help you understand how CPUs and low-level computer architecture work by simulating a basic computer system from scratch.

## Project Goals
- Learn the fundamentals of CPU and VM design
- Explore instruction sets, registers, memory, and program execution
- Provide a clear, well-documented codebase for experimentation and learning

## Features
- 16-bit word size for all operations and memory
- General-purpose and special-purpose registers (PC, SP, FLAGS, etc.)
- Simple instruction set (arithmetic, logic, memory access, control flow)
- Memory-mapped I/O and bus abstraction
- Halt and error handling for safe execution

## Architecture Overview

The VM consists of:
- **Registers:** 6 registers (R0–R3 general-purpose, RPC program counter RIR)
- **Memory:** Linear address space, 16-bit words
- **Instruction Set:** Each instruction is 16 bits, with 4 bits for the opcode and the rest for operands
- **Execution Loop:** Fetch-decode-execute cycle, halts on errors or HALT instruction

![VM Architecture](https://stephengream.com/static/ab42d86625f01dc6f42bc7d10e4c7326/7c2a6/CPU_v0.1.png)

## 16-bit Instruction Format

Each instruction is 16 bits, divided as follows:

> In my implementation, each register’s value can hold either actual data from memory or a memory address, depending on the OPCODE being used.

| Bits         | Field         | Description                                 |
|------------- |---------------|---------------------------------------------|
| 15–12 (4)    | Opcode        | Operation to perform                        |
| 11–8  (4)    | Register 1    | Destination register                        |
| 7–4   (4)    | Register 2    | Source register                             |
| 3–0   (4)    | Imm/Offset    | Immediate value or offset (opcode-dependent)|

#### Visual Representation

```
┌───────┬────────────┬────────────┬────────────┐
│Opcode │ Register 1 │ Register 2 │ Imm/Offset │
└───────┴────────────┴────────────┴────────────┘
 4 bits  4 bits       4 bits       4 bits
```

* The meaning of the last 4 bits (Imm/Ofs) depends on the opcode: it can be a small constant or an offset.
  
## Example Usage

Build the project:
```sh
cargo build
```

Run the VM (example, see `main.rs` for entry point):
```sh
cargo run
```

## Future Improvements plans for the 16-bit VM

Implement runtime

Run eBPF-inspired programs: Support a small subset of eBPF instructions (arithmetic, logic, memory access, branching) adapted to 16-bit registers.

Enhanced stack and memory: Add bounds-checked stack and simple memory model to handle eBPF-like program execution safely.

Syscalls / helpers: Implement basic runtime functions such as logging or debug output for program interaction.

Instruction decoding and execution: Support immediate values, relative jumps, and conditional branching for richer eBPF-style logic.

Debugging and verification: Add execution logs, stack/register inspection, and basic safety checks (overflow, invalid jumps).

## Resources & Inspiration
- [Writing an LC-3 VM in C](https://www.jmeiners.com/lc3-vm/)
- [Writing a VM (Stephen Gream)](https://stephengream.com/writing-a-vm-part-one/)
- [Build Virtual Machine](https://www.youtube.com/watch?v=OjaAToVkoTw&list=PLSewtCfzWXPKemmD0_hi_G8ZOvYxSJDpo&index=10)

---
This project is for learning and experimentation. Contributions and questions are welcome!