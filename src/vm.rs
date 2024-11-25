use crate::opcodes::{Argument, Opcode, TrapCode};
use crate::util::join_u8;
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::io;
use std::io::{Read, Write};

const MEMORY_SIZE: usize = 2_usize.pow(16);
const REG_IDX_PC: usize = 8;
const REG_IDX_COND: usize = 9;
const PC_START_POS: usize = 0x3000;
const REG_RET: usize = 7;

const MR_KBSR: usize = 0xFE00; // Keyboard Status memory mapping
const MR_KBDR: usize = 0xFE02; // Keyboard Data memory mapping

#[derive(PartialEq, Eq)]
enum ConditionFlag {
    Pos,
    Neg,
    Zero,
    None,
}

pub struct VM {
    running: bool,
    registers: [u16; 10],
    memory: [u16; MEMORY_SIZE],
}

impl VM {
    pub(crate) fn new(data: &[u8]) -> Self {
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

    pub(crate) fn run(&mut self) {
        self.running = true;

        while self.running {
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
            dbg!(&self);
        }
    }

    fn fetch(&self) -> u16 {
        *self.memory.get(self.pc()).expect("Out of bounds fetch")
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
        eprintln!("Loading data at origin {:#x}", origin);
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
            } => {
                let res = u16::wrapping_add(self.registers[sr1], self.registers[sr2]);
                eprintln!("ADD reg[{dr}] <- reg[{sr1}] + reg[{sr2}] = {res}");
                self.registers[dr] = res;
                self.set_flags(res as i16);
            }
            Opcode::ADD {
                dr,
                sr1,
                sr2: Argument::Immediate(val),
            } => {
                let res = u16::wrapping_add(self.registers[sr1], val as u16);
                eprintln!("ADD reg[{dr}] <- reg[{sr1}] + {val} = {res}");
                self.registers[dr] = res;
                self.set_flags(res as i16);
            }
            Opcode::AND {
                dr,
                sr1,
                sr2: Argument::Reg(sr2),
            } => {
                let res = self.registers[sr1] & self.registers[sr2];
                eprintln!(
                    "AND reg[{}] <- reg[{}] & reg[{}] = {:#0x}",
                    dr, sr1, sr2, res
                );
                self.registers[dr] = res;
                self.set_flags(res as i16);
            }
            Opcode::AND {
                dr,
                sr1,
                sr2: Argument::Immediate(val),
            } => {
                let res = ((self.registers[sr1] as i16) & val) as u16;
                eprintln!(
                    "AND reg[{}] <- reg[{}] & {:#0x} = {:#0x}",
                    dr, sr1, val, res
                );
                self.registers[dr] = res;
                self.set_flags(res as i16);
            }
            Opcode::BR { n, z, p, offset } => {
                let actual_flag = self.cond_flag();
                match ((n, z, p), actual_flag) {
                    ((true, _, _), ConditionFlag::Neg)
                    | ((_, true, _), ConditionFlag::Zero)
                    | ((_, _, true), ConditionFlag::Pos) => {
                        eprintln!("BR: Taken, n={n}, z={z}, p={p} | offset = {offset}");
                        self.set_pc(self.pc_with_offset(offset));
                    }
                    _ => eprintln!("BR: Not Taken, n={n}, z={z}, p={p} | offset = {offset}"),
                }
            }
            Opcode::JMP { base_r } => {
                eprintln!("JMP {:#0x}", base_r);
                self.set_pc(self.registers[base_r] as usize);
            }
            Opcode::RET => {
                let dir = self.registers[REG_RET] as usize;
                eprintln!("RET {:#0x}", dir);
                self.set_pc(dir);
            }
            Opcode::JSR { offset } => {
                self.registers[REG_RET] = self.pc() as u16;
                let dir = self.pc_with_offset(offset);
                eprintln!("JSR {:#0x}+{} = {:#0x}", self.pc(), offset, dir);
                self.set_pc(dir);
            }
            Opcode::JSRR { base_r } => {
                self.registers[REG_RET] = self.pc() as u16;
                eprintln!("JSRR {:#0x}", self.registers[base_r]);
                self.set_pc(self.registers[base_r] as usize);
            }
            Opcode::LD { dr, offset } => {
                let res = self.read_with_offset(offset);
                eprintln!("LD reg[{dr}] <- {res}");
                self.registers[dr] = res;
                self.set_flags(res as i16);
            }
            Opcode::LDI { dr, offset } => {
                let dir = self.read_with_offset(offset) as usize;
                let res = self.read(dir);
                eprintln!(
                    "LDI reg[{}] <- mem[{:#0x}+{}={:#0x}] = {:#0x}",
                    dr,
                    self.pc(),
                    offset,
                    dir,
                    res
                );
                self.registers[dr] = res;
                self.set_flags(res as i16);
            }
            Opcode::LDR { dr, base_r, offset } => {
                let base_r_dir = self.registers[base_r];
                let dir = (base_r_dir as i16).wrapping_add(offset) as usize;
                let res = self.read(dir);
                eprintln!(
                    "LDR reg[{}] <- mem[{:#0x}+{}={:#0x}] = {:#0x}",
                    dr, base_r, offset, dir, res
                );
                self.registers[dr] = res;
                self.set_flags(res as i16);
            }
            Opcode::LEA { dr, offset } => {
                let dir = self.pc_with_offset(offset) as u16;
                eprintln!("LEA reg[{}] <- {:#0x}", dr, dir);
                self.registers[dr] = dir;
                self.set_flags(dir as i16);
            }
            Opcode::NOT { dr, sr } => {
                let res = !self.registers[sr];
                eprintln!("NOT reg[{}] <- !reg[{}] = {:#0x}", dr, sr, res);
                self.registers[dr] = res;
                self.set_flags(res as i16);
            }
            Opcode::RTI => {
                eprintln!("RTI");
                dbg!(&opcode);
            }
            Opcode::ST { sr, offset } => {
                let dir = self.pc_with_offset(offset);
                let val = self.registers[sr];
                eprintln!(
                    "ST mem[{:#0x}+{:#0x} = {:#0x}] <- reg[{}] = {:#0x}",
                    self.pc(),
                    offset,
                    dir,
                    sr,
                    val
                );
                self.memory[dir] = val;
            }
            Opcode::STI { sr, offset } => {
                let dir = self.read_with_offset(offset) as usize;
                let val = self.registers[sr];
                eprintln!("STI mem[{:#0x}] <- reg[{}] = {:#0x}", dir, sr, val);
                self.memory[dir] = self.registers[sr];
            }
            Opcode::STR { sr, base_r, offset } => {
                let base_r_dir = self.registers[base_r];
                let dir = (base_r_dir as i16).wrapping_add(offset) as usize;
                let val = self.registers[sr];
                eprintln!("STR mem[{:#0x}] <- reg[{}] = {:#0x}", dir, sr, val);
                self.memory[dir] = val;
            }
            Opcode::TRAP { trap_code } => {
                eprintln!("TRAP {:?}", trap_code);
                self.registers[REG_RET] = self.pc() as u16;
                self.handle_trap_code(trap_code);
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
        let new_pc = u16::try_from(new_pc).expect("PC should fit in 16 bits");
        self.registers[REG_IDX_PC] = new_pc;
    }

    fn advance_pc(&mut self) {
        self.set_pc(self.pc_with_offset(1));
    }

    fn pc_with_offset(&self, offset: i16) -> usize {
        let pc = self.pc() as u16;
        pc.wrapping_add_signed(offset) as usize
    }

    fn read(&mut self, position: usize) -> u16 {
        if position == MR_KBSR {
            self.handle_keyboard();
        }
        *self.memory.get(position).expect("Out of bounds read")
    }

    fn read_with_offset(&mut self, offset: i16) -> u16 {
        self.read(self.pc_with_offset(offset))
    }

    fn handle_keyboard(&mut self) {
        let mut buffer = [0; 1];
        io::stdin().read_exact(&mut buffer).unwrap();
        if buffer[0] != 0 {
            self.memory[MR_KBSR] = 1 << 15;
            self.memory[MR_KBDR] = buffer[0] as u16;
        } else {
            self.memory[MR_KBSR] = 0;
        }
    }

    fn set_flags(&mut self, res: i16) {
        let cond = match res.cmp(&0i16) {
            Ordering::Less => ConditionFlag::Neg,
            Ordering::Equal => ConditionFlag::Zero,
            Ordering::Greater => ConditionFlag::Pos,
        };
        self.set_cond_flag(cond);
    }

    fn handle_trap_code(&mut self, trap_code: TrapCode) {
        match trap_code {
            TrapCode::Getc => {
                let mut buffer = [0; 1];
                io::stdin().read_exact(&mut buffer).unwrap();
                self.registers[0] = buffer[0] as u16;
                self.set_flags(self.registers[0] as i16);
            }
            TrapCode::Out => {
                let ch = self.registers[0] as u8;
                print!("{}", ch as char);
                eprint!("{}", ch as char);
                io::stdout().flush().expect("Failed to flush");
            }
            TrapCode::Puts => {
                let mut i = self.registers[0] as usize;
                while self.memory[i] != 0x0000 {
                    let ch = self.memory[i] as u8;
                    print!("{}", ch as char);
                    eprint!("{}", ch as char);
                    i += 1;
                }
                io::stdout().flush().expect("Failed to flush");
            }
            TrapCode::In => {
                println!("Enter a character: ");
                io::stdout().flush().expect("failed to flush");
                let char = io::stdin()
                    .bytes()
                    .next()
                    .and_then(|result| result.ok())
                    .map(|byte| byte as u16)
                    .unwrap();
                self.registers[0] = char;
                self.set_flags(self.registers[0] as i16);
            }
            TrapCode::Putsp => {
                let mut i = self.registers[0] as usize;
                while self.memory[i] != 0x0000 {
                    let ch = self.memory[i];
                    let (ch1, ch2) = (ch & 0xFF, ch >> 8);
                    print!("{}", (ch1 as u8) as char);
                    eprint!("{}", (ch1 as u8) as char);
                    if ch2 != 0x00 {
                        print!("{}", (ch2 as u8) as char);
                        eprint!("{}", (ch2 as u8) as char);
                    }
                    i += 1;
                }
            }
            TrapCode::Halt => {
                self.running = false;
            }
        }
    }
}

impl Debug for VM {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "[r0={},r1={},r2={},r3={},r4={},r5={},r6={},r7={},PC={:#X},COND={:#X}]",
            self.registers[0],
            self.registers[1],
            self.registers[2],
            self.registers[3],
            self.registers[4],
            self.registers[5],
            self.registers[6],
            self.registers[7],
            self.registers[8],
            self.registers[9]
        )
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

    #[test]
    fn add_register_simple() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.registers[0] = 10;
        vm.registers[1] = 3;
        vm.registers[2] = 5;

        vm.execute(Opcode::ADD {
            dr: 0,
            sr1: 1,
            sr2: Argument::Reg(2),
        });
        assert_eq!(vm.registers[0], 8);
        assert_eq!(vm.registers[1], 3);
        assert_eq!(vm.registers[2], 5);
    }

    #[test]
    fn add_register_negative() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.registers[0] = 0;
        vm.registers[1] = 0;
        vm.registers[2] = 0;

        vm.execute(Opcode::ADD {
            dr: 0,
            sr1: 1,
            sr2: Argument::Immediate(-5),
        });
        assert_eq!(vm.registers[0], 0b1111_1111_1111_1011);
        assert_eq!(vm.registers[1], 0);
        assert_eq!(vm.registers[2], 0);
    }

    #[test]
    fn add_reg_overflow() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.registers[0] = 3;
        vm.registers[1] = u16::MAX;
        vm.registers[2] = 2;

        vm.execute(Opcode::ADD {
            dr: 0,
            sr1: 1,
            sr2: Argument::Reg(2),
        });
        assert_eq!(vm.registers[0], 1);
        assert_eq!(vm.registers[1], u16::MAX);
        assert_eq!(vm.registers[2], 2);
    }

    #[test]
    fn add_imm5_overflow() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.registers[0] = 3;
        vm.registers[1] = u16::MAX;
        vm.registers[2] = 1;

        vm.execute(Opcode::ADD {
            dr: 0,
            sr1: 1,
            sr2: Argument::Immediate(2),
        });
        assert_eq!(vm.registers[0], 1);
        assert_eq!(vm.registers[1], u16::MAX);
        assert_eq!(vm.registers[2], 1);
    }

    #[test]
    fn execute_and_regs() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.registers[0] = 3;
        vm.registers[1] = 4;
        vm.registers[2] = 7;

        vm.execute(Opcode::AND {
            dr: 0,
            sr1: 1,
            sr2: Argument::Reg(1),
        });
        assert_eq!(vm.registers[0], 4 & 7);
        assert_eq!(vm.registers[1], 4);
        assert_eq!(vm.registers[2], 7);
    }

    #[test]
    fn execute_and_imm() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.registers[0] = 3;
        vm.registers[1] = 4;
        vm.registers[2] = 7;

        vm.execute(Opcode::AND {
            dr: 0,
            sr1: 1,
            sr2: Argument::Immediate(9),
        });
        assert_eq!(vm.registers[0], 4 & 9);
        assert_eq!(vm.registers[1], 4);
        assert_eq!(vm.registers[2], 7);
    }

    #[test]
    fn execute_and_zero() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.registers[0] = 3;
        vm.registers[1] = 4;
        vm.registers[2] = 7;

        vm.execute(Opcode::AND {
            dr: 0,
            sr1: 1,
            sr2: Argument::Immediate(0),
        });
        assert_eq!(vm.registers[0], 0);
        assert_eq!(vm.registers[1], 4);
        assert_eq!(vm.registers[2], 7);
    }

    #[test]
    fn execute_br_not_taken() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        assert_eq!(vm.pc(), 0x3000);
        vm.set_cond_flag(ConditionFlag::Neg);
        vm.execute(Opcode::BR {
            n: false,
            z: false,
            p: true,
            offset: 15,
        });
        assert_eq!(vm.pc(), 0x3000);
    }

    #[test]
    fn execute_br_taken_pos() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        assert_eq!(vm.pc(), 0x3000);
        vm.set_cond_flag(ConditionFlag::Pos);
        vm.execute(Opcode::BR {
            n: false,
            z: false,
            p: true,
            offset: 15,
        });
        assert_eq!(vm.pc(), 0x3000 + 15);
    }

    #[test]
    fn execute_br_taken_neg() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        assert_eq!(vm.pc(), 0x3000);
        vm.set_cond_flag(ConditionFlag::Pos);
        vm.execute(Opcode::BR {
            n: false,
            z: false,
            p: true,
            offset: -15,
        });
        assert_eq!(vm.pc(), 0x3000 - 15);
    }

    #[test]
    fn execute_br_taken_overflow() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.set_pc(u16::MAX as usize);
        assert_eq!(vm.pc(), u16::MAX as usize);
        vm.set_cond_flag(ConditionFlag::Pos);
        vm.execute(Opcode::BR {
            n: false,
            z: false,
            p: true,
            offset: 2,
        });
        assert_eq!(vm.pc(), 1);
    }

    #[test]
    fn execute_br_taken_underflow() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.set_pc(0);
        assert_eq!(vm.pc(), 0);
        vm.set_cond_flag(ConditionFlag::Pos);
        vm.execute(Opcode::BR {
            n: false,
            z: false,
            p: true,
            offset: -1,
        });
        assert_eq!(vm.pc(), u16::MAX as usize);
    }

    #[test]
    fn execute_jmp() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.registers[1] = 0x3999;
        vm.execute(Opcode::JMP { base_r: 1 });
        assert_eq!(vm.pc(), 0x3999);
    }

    #[test]
    fn execute_ret() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.registers[7] = 0x3999;
        vm.execute(Opcode::RET);
        assert_eq!(vm.pc(), 0x3999);
    }

    #[test]
    fn execute_jsr_pos() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.set_pc(0x3005);
        vm.execute(Opcode::JSR { offset: 15 });
        assert_eq!(vm.pc(), 0x3005 + 15);
        assert_eq!(vm.registers[REG_RET], 0x3005);
    }

    #[test]
    fn execute_jsr_neg() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.set_pc(0x3005);
        vm.execute(Opcode::JSR { offset: -15 });
        assert_eq!(vm.pc(), 0x3005 - 15);
        assert_eq!(vm.registers[REG_RET], 0x3005);
    }

    #[test]
    fn execute_jsrr() {
        let data: Vec<u8> = vec![0x30, 0x00, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.set_pc(0x3005);
        vm.registers[2] = 0x4000;
        vm.execute(Opcode::JSRR { base_r: 2 });
        assert_eq!(vm.pc(), 0x4000);
        assert_eq!(vm.registers[REG_RET], 0x3005);
    }

    #[test]
    fn execute_ld_pos() {
        let data: Vec<u8> = vec![0x30, 0x00, 0x10, 0x10, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.execute(Opcode::LD { dr: 0, offset: 2 });
        assert_eq!(vm.registers[0], 0xbabe);
    }

    #[test]
    fn execute_ld_neg() {
        let data: Vec<u8> = vec![0x30, 0x00, 0x10, 0x10, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.set_pc(0x3000 + 2);
        vm.execute(Opcode::LD { dr: 0, offset: -1 });
        assert_eq!(vm.registers[0], 0xcafe);
    }

    #[test]
    fn execute_ldi_pos() {
        let data: Vec<u8> = vec![0x30, 0x00, 0x30, 0x02, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.execute(Opcode::LDI { dr: 0, offset: 0 });
        assert_eq!(vm.registers[0], 0xbabe);
    }

    #[test]
    fn execute_ldi_neg() {
        let data: Vec<u8> = vec![0x30, 0x00, 0x30, 0x02, 0xca, 0xfe, 0xba, 0xbe];
        let mut vm = VM::new(&data);

        vm.set_pc(0x3000 + 1);
        vm.execute(Opcode::LDI { dr: 0, offset: -1 });
        assert_eq!(vm.registers[0], 0xbabe);
    }
}
