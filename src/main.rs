#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::upper_case_acronyms)]

mod opcodes;

use crate::opcodes::Argument;
use crate::opcodes::Opcode;
use std::cmp::PartialEq;
use std::fs;

const MEMORY_SIZE: usize = 2_usize.pow(16);
const REG_IDX_PC: usize = 8;
const REG_IDX_COND: usize = 9;
const PC_START_POS: usize = 0x3000;
const REG_RET: usize = 7;

#[derive(PartialEq, Eq)]
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
            dbg!(&self.registers);
        }
    }

    fn fetch(&self) -> u16 {
        *self
            .memory
            .get(self.pc())
            .expect("Out of bounds fetch")
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

    fn cond_flag_any_set(&self) -> bool {
        self.cond_flag() != ConditionFlag::None
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
                dr,
                sr1,
                sr2: Argument::Reg(sr2),
            } => self.registers[dr] = self.registers[sr1] + self.registers[sr2],
            Opcode::ADD {
                dr,
                sr1,
                sr2: Argument::Immediate(val),
            } => {
                self.registers[dr] = (self.registers[sr1] as i16).wrapping_add(val) as u16;
            }
            Opcode::AND {
                dr,
                sr1,
                sr2: Argument::Reg(sr2),
            } => self.registers[dr] = self.registers[sr1] & self.registers[sr2],
            Opcode::AND {
                dr,
                sr1,
                sr2: Argument::Immediate(val),
            } => self.registers[dr] = ((self.registers[sr1] as i16) & val) as u16,
            Opcode::BR { n, z, p, offset } => {
                if self.cond_flag_any_set() {
                    self.set_pc(self.pc_with_offset(offset));
                }
            }
            Opcode::JMP { base_r } => {
                self.set_pc(self.registers[base_r] as usize);
            }
            Opcode::RET => {
                self.set_pc(self.registers[REG_RET] as usize);
            }
            Opcode::JSR { offset } => {
                self.set_pc(self.pc_with_offset(offset));
            }
            Opcode::JSRR { base_r } => {
                self.set_pc(self.registers[base_r] as usize);
            }
            Opcode::LD { dr, offset } => self.registers[dr] = self.read_with_offset(offset),
            Opcode::LDI { dr, offset } => {
                let dir = self.read_with_offset(offset) as usize;
                self.registers[dr] = self.read(dir);
            }
            Opcode::LDR { dr, base_r, offset } => {
                let dir = base_r_with_offset(base_r, offset);
                self.registers[dr] = self.read(dir);
            }
            Opcode::LEA { dr, offset } => {
                self.registers[dr] = self.pc_with_offset(offset) as u16;
            }
            Opcode::NOT { dr, sr } => {
                self.registers[dr] = !sr as u16;
            }
            Opcode::RTI => {
                dbg!(&opcode);
            }
            Opcode::ST { sr, offset } => {
                self.memory[self.pc_with_offset(offset)] = self.registers[sr];
            }
            Opcode::STI { sr, offset } => {
                let dir = self.read_with_offset(offset) as usize;
                self.memory[dir] = self.registers[sr];
            }
            Opcode::STR { sr, base_r, offset } => {
                let dir = base_r_with_offset(base_r, offset);
                self.memory[dir] = self.registers[sr];
            }
            Opcode::TRAP { trap_vec } => {
                self.registers[REG_RET] = self.pc() as u16;
                self.set_pc(self.memory[trap_vec as usize] as usize);
            }
            Opcode::RESERVED => {
                dbg!(&opcode);
                panic!("Reserved Instruction");
            }
        }
    }

    fn pc(&self) -> usize {
        self.registers[REG_IDX_PC] as usize
    }

    fn set_pc(&mut self, new_pc: usize) {
        self.registers[REG_IDX_PC] = new_pc as u16;
    }

    fn advance_pc(&mut self) {
        self.registers[REG_IDX_PC] += 1;
    }

    fn pc_with_offset(&self, offset: i16) -> usize {
        (self.pc() as i16).wrapping_add(offset) as usize // TODO is this ok?
    }

    fn read(&self, position: usize) -> u16 {
        *self.memory.get(position).expect("Out of bounds read")
    }

    fn read_with_offset(&self, offset: i16) -> u16 {
        self.read(self.pc_with_offset(offset))
    }
}

fn base_r_with_offset(base_r: usize, offset: i16) -> usize {
    (base_r as i16).wrapping_add(offset) as usize
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
        assert_eq!(vm.memory[0x3000], 0xcafe);
        assert_eq!(vm.memory[0x3001], 0xbabe);
    }
}
