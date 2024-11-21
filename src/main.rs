#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::upper_case_acronyms)]

mod opcodes;

use crate::opcodes::Argument;
use crate::opcodes::Opcode;
use std::fs;

const MEMORY_SIZE: usize = 2_usize.pow(16);
const REG_IDX_PC: usize = 8;
const REG_IDX_COND: usize = 9;
const PC_START_POS: u16 = 0x3000;

enum ConditionFlag {
    Pos,
    Neg,
    Zero,
    None,
}

struct VM {
    running: bool,
    registers: [u16; 10],
    memory: [u16; MEMORY_SIZE],
}

impl VM {
    fn new(data: &[u8]) -> Self {
        let mut vm = VM {
            running: false,
            registers: [0; 10],
            memory: [0; MEMORY_SIZE],
        };
        vm.read_data_into_memory(data);
        vm.set_pc(PC_START_POS);
        vm.set_cond_flag(ConditionFlag::None);
        vm
    }

    fn run(&mut self) {
        self.running = true;
        let mut cycle_count = 0; // For debug purposes
        while self.running && cycle_count < 30 {
            // Fetch
            let instruction = self.fetch();
            self.advance_pc();

            // Decode
            let Ok(opcode) = Opcode::try_from(instruction) else {
                println!("ERR: {instruction} not recognized!");
                self.running = false;
                break;
            };

            // Execute
            self.execute(opcode);
            cycle_count += 1;
        }
    }

    fn fetch(&self) -> u16 {
        *self
            .memory
            .get(self.pc() as usize)
            .expect("Out of bounds fetch")
    }

    fn pc(&self) -> u16 {
        self.registers[REG_IDX_PC]
    }

    fn set_pc(&mut self, new_pc: u16) {
        self.registers[REG_IDX_PC] = new_pc;
    }

    fn advance_pc(&mut self) {
        self.registers[REG_IDX_PC] += 1;
    }

    fn cond_flag(&self) -> ConditionFlag {
        let r = self.registers[REG_IDX_COND];
        if r == 1 << 0 {
            ConditionFlag::Pos
        } else if r == 1 << 1 {
            ConditionFlag::Zero
        } else if r == 1 << 2 {
            ConditionFlag::Neg
        } else {
            ConditionFlag::None
        }
    }

    fn set_cond_flag(&mut self, cond: ConditionFlag) {
        self.registers[REG_IDX_COND] = match cond {
            ConditionFlag::Pos => 1 << 0,
            ConditionFlag::Zero => 1 << 1,
            ConditionFlag::Neg => 1 << 2,
            ConditionFlag::None => 0,
        }
    }

    fn read_data_into_memory(&mut self, data: &[u8]) {
        let origin = join_u8(data[0], data[1]) as usize;
        println!("Loading data at origin {:#x}", origin);
        let mut mem_i: usize = origin;
        let mut data_i: usize = 2; // Skip the origin
        while mem_i < MEMORY_SIZE - 1 && data_i < data.len() {
            let word = join_u8(data[data_i], data[data_i + 1]);
            self.memory[mem_i] = word;
            mem_i += 1;
            data_i += 2;
        }
    }

    fn execute(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::ADD {
                dr: dr,
                sr1: sr1,
                sr2: Argument::Reg(reg),
            } => {
                dbg!(&opcode);
            }
            Opcode::ADD {
                dr,
                sr1,
                sr2: Argument::Immediate(val),
            } => {
                dbg!(&opcode);
            }
            Opcode::AND {
                dr,
                sr1,
                sr2: Argument::Reg(reg),
            } => {
                dbg!(&opcode);
            }
            Opcode::AND {
                dr,
                sr1,
                sr2: Argument::Immediate(val),
            } => {
                dbg!(&opcode);
            }
            Opcode::BR { n, z, p, offset } => {
                dbg!(&opcode);
            }
            Opcode::JMP { base_r } => {
                dbg!(&opcode);
            }
            Opcode::RET => {
                dbg!(&opcode);
            }
            Opcode::JSR { offset } => {
                dbg!(&opcode);
            }
            Opcode::JSRR => {
                dbg!(&opcode);
            }
            Opcode::LD { dr, offset } => {
                dbg!(&opcode);
            }
            Opcode::LDI { dr, offset } => {
                dbg!(&opcode);
            }
            Opcode::LDR { dr, base_r, offset } => {
                dbg!(&opcode);
            }
            Opcode::LEA { dr, offset } => {
                dbg!(&opcode);
            }
            Opcode::NOT { dr, sr } => {
                dbg!(&opcode);
            }
            Opcode::RTI => {
                dbg!(&opcode);
            }
            Opcode::ST { sr, offset } => {
                dbg!(&opcode);
            }
            Opcode::STI { sr, offset } => {
                dbg!(&opcode);
            }
            Opcode::STR { sr, base_r, offset } => {
                dbg!(&opcode);
            }
            Opcode::TRAP { trap_vec } => {
                dbg!(&opcode);
            }
            Opcode::RESERVED => {
                dbg!(&opcode);
            }
        }
    }
}

// I'm supposed to swap endianness according to docs but so far it was working,
// I'll check later if I run into problems.
fn join_u8(hi: u8, lo: u8) -> u16 {
    let hi = hi as u16;
    let lo = lo as u16;
    (hi << 8) | lo
}

fn main() {
    println!("lc3-vm");
    let data: Vec<u8> = fs::read("2048.obj").expect("Failed to load file");
    let mut vm = VM::new(&data);
    vm.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_le_to_be() {
        let data: Vec<u8> = vec![0xca, 0xfe];
        let res = join_u8(data[0], data[1]);
        assert_eq!(0xcafe, res);
    }

    #[test]
    fn test_read_data_in_le() {
        // Memory offset is 0x3000
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let vm = VM::new(&data);
        for i in 0x3000_usize..0x3000 + 4 {
            println!("{:x?}", vm.memory[i]);
        }
        for i in 0..0x3000 {
            assert_eq!(vm.memory[i], 0);
        }
        assert_eq!(memory[0x3000], 0xcafe);
        assert_eq!(memory[0x3001], 0xbabe);
    }
}
