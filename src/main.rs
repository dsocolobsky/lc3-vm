#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::upper_case_acronyms)]

mod opcodes;

use std::fs;
use crate::opcodes::Opcode;

const MEMORY_SIZE: usize = 2_usize.pow(16);
const REG_IDX_PC: usize = 8;
const REG_IDX_COND: usize = 9;

enum ConditionFlag {
    Pos,
    Neg,
    Zero,
    None
}

struct VM {
    registers: [u16; 10],
}

impl VM {
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
}

// I'm supposed to swap endianness according to docs but so far it was working,
// I'll check later if I run into problems.
fn join_u8(hi: u8, lo: u8) -> u16 {
    let hi = hi as u16;
    let lo = lo as u16;
    (hi << 8) | lo
}

fn read_data_into_memory(data: &[u8], memory: &mut [u16]) {
    let origin = join_u8(data[0], data[1]) as usize;
    println!("Loading data at origin {:#x}", origin);
    let mut mem_i: usize = origin;
    let mut data_i: usize = 2; // Skip the origin
    while mem_i < MEMORY_SIZE - 1 && data_i < data.len() {
        let word = join_u8(data[data_i], data[data_i + 1]);
        memory[mem_i] = word;
        mem_i += 1;
        data_i += 2;
    }
}

fn main() {
    let mut memory: [u16; MEMORY_SIZE] = [0; MEMORY_SIZE];
    println!("lc3-vm");
    let data: Vec<u8> = fs::read("2048.obj").expect("Failed to load file");
    read_data_into_memory(&data, &mut memory);

    let mut pc: usize = 0x3000;
    while pc < 0x3000 + 20 {
        // Arbitrary limit to debug for now
        let op = Opcode::try_from(memory[pc]).expect("Unknown opcode");
        dbg!(op);
        pc += 1;
    }
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
        let mut memory: [u16; MEMORY_SIZE] = [0; MEMORY_SIZE];
        read_data_into_memory(&data, &mut memory);
        for i in 0x3000_usize..0x3000 + 4 {
            println!("{:x?}", memory[i]);
        }
        for i in 0..0x3000 {
            assert_eq!(memory[i], 0);
        }
        assert_eq!(memory[0x3000], 0xcafe);
        assert_eq!(memory[0x3001], 0xbabe);
    }
}
