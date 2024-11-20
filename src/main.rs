#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::upper_case_acronyms)]
use std::fs;

const MEMORY_SIZE: usize = 2_usize.pow(16);

#[derive(Debug)]
enum Argument {
    Reg(u16),
    Immediate(i16),
}

#[derive(Debug)]
enum Opcode {
    ADD {
        dr: u16,
        sr1: u16,
        sr2: Argument,
    }, // Add (dr <- sr1 + sr2/imm5)
    AND {
        dr: u16,
        sr1: u16,
        sr2: Argument,
    }, // Bitwise And (dr <- sr1 & sr2/imm5)
    BR {
        n: bool,
        z: bool,
        p: bool,
        offset: i16,
    }, // Branch (If n/z/p => pc += offset)
    JMP {
        base_r: u16,
    }, // Jump (pc <- base_r)
    RET, // Return from subroutine (JMP) (pc <- r7)
    JSR {
        offset: i16,
    }, // Jump To Subroutine (pc <- pc + offset)
    JSRR, // Jump to Subroutine (pc <- base_r)
    LD {
        dr: u16,
        offset: i16,
    }, // Load Direct from memory (dr <- mem[pc+offset])
    LDI {
        dr: u16,
        offset: i16,
    }, // Load Indirect from memory (dr <- mem[mem[pc+offset]])
    LDR {
        dr: u16,
        base_r: u16,
        offset: i16,
    }, // Load Base+Offset (dr <- mem[base_r + offset])
    LEA {
        dr: u16,
        offset: i16,
    }, // Load Effective Address (dr <- pc + offset)
    NOT {
        dr: u16,
        sr: u16,
    }, // Bitwise Not (dr <- ~sr)
    RTI, // Return From Interrupt
    ST {
        sr: u16,
        offset: i16,
    }, // Store (mem[pc+offset] <- sr)
    STI {
        sr: u16,
        offset: i16,
    }, // Store Indirect (mem[mem[pc+offset]] <- sr)
    STR {
        sr: u16,
        base_r: u16,
        offset: i16,
    }, // Store Base+Offset (mem[sr+offset] <- sr)
    TRAP { trap_vec: u16 }, // Execute Trap
    RESERVED, // Unused. Throws an Illegal Opcode Exception.
}

fn decode_instruction(instruction: u16) -> Option<Opcode> {
    let op = (instruction >> 12) as u8; // Highest 4 bits
    match op {
        0b0001 => {
            // ADD
            let dr = instruction & 0b0000_111_000_0_00_000 >> 9;
            let sr1 = instruction & 0b0000_000_111_0_00_000 >> 6;
            if instruction & (1 << 5) == 0 {
                // Use sr2 register as 2nd argument
                let sr2 = instruction & 0b0000_000_000_0_00_111;
                Some(Opcode::ADD {
                    dr,
                    sr1,
                    sr2: Argument::Reg(sr2),
                })
            } else {
                // Use imm5 as 2nd argument
                let imm5 = instruction & 0b0000_000_000_0_11_111;
                Some(Opcode::ADD {
                    dr,
                    sr1,
                    sr2: Argument::Immediate(imm5 as i16),
                })
            }
        }
        0b0101 => {
            // AND , this is the same decode case as ADD
            let dr = instruction & 0b0000_111_000_0_00_000 >> 9;
            let sr1 = instruction & 0b0000_000_111_0_00_000 >> 6;
            if instruction & (1 << 5) == 0 {
                // Use sr2 register as 2nd argument
                let sr2 = instruction & 0b0000_000_000_0_00_111;
                Some(Opcode::AND {
                    dr,
                    sr1,
                    sr2: Argument::Reg(sr2),
                })
            } else {
                // Use imm5 as 2nd argument
                let imm5 = instruction & 0b0000_000_000_0_11_111;
                Some(Opcode::AND {
                    dr,
                    sr1,
                    sr2: Argument::Immediate(imm5 as i16),
                })
            }
        }
        0b0000 => {
            // BR  Conditional Branch
            Some(Opcode::BR {
                n: instruction & (1 << 11) != 0,
                z: instruction & (1 << 10) != 0,
                p: instruction & (1 << 9) != 0,
                offset: (instruction & 0b1_1111_1111) as i16,
            })
        }
        0b1100 => {
            // JMP/RET
            let base_r = instruction & 0b0000_000_111_0_00_000 >> 6;
            if base_r == 0b111 {
                Some(Opcode::RET)
            } else {
                Some(Opcode::JMP { base_r })
            }
        }
        0b0100 => {
            // JSR/JSRR
            if instruction & (1 << 11) != 0 {
                Some(Opcode::JSR {
                    offset: (instruction & 0b111_11111) as i16,
                })
            } else {
                Some(Opcode::JSRR)
            }
        }
        0b0010 => {
            // LD
            Some(Opcode::LD {
                dr: (instruction & 0b111_0_00000000) >> 9,
                offset: (instruction & 0b1111_1111) as i16,
            })
        }
        0b1010 => {
            // LDI , decode same as LD
            Some(Opcode::LDI {
                dr: (instruction & 0b111_0_00000000) >> 9,
                offset: (instruction & 0b1111_1111) as i16,
            })
        }
        0b0110 => {
            // LDR
            Some(Opcode::LDR {
                dr: (instruction & 0b111_000_000000) >> 9,
                base_r: (instruction & 0b000_111_000000) >> 6,
                offset: (instruction & 0b000_000_111111) as i16,
            })
        }
        0b1110 => {
            // LEA
            Some(Opcode::LEA {
                dr: (instruction & 0b111_0000_0000) >> 8,
                offset: (instruction & 0b000_1111_1111) as i16,
            })
        }
        0b1001 => {
            // NOT
            Some(Opcode::NOT {
                dr: (instruction & 0b111_000_0_00000) >> 9,
                sr: (instruction & 0b111_000_0_00000) >> 9,
            })
        }
        0b1000 => Some(Opcode::RTI),
        0b0011 => {
            // ST
            Some(Opcode::ST {
                sr: (instruction & 0b111_0000_0000) >> 8,
                offset: (instruction & 0b000_1111_1111) as i16,
            })
        }
        0b1011 => {
            // STI
            Some(Opcode::STI {
                sr: (instruction & 0b111_0000_0000) >> 8,
                offset: (instruction & 0b000_1111_1111) as i16,
            })
        }
        0b0111 => {
            // STR
            Some(Opcode::STR {
                sr: (instruction & 0b111_000_000000) >> 9,
                base_r: (instruction & 0b000_111_000000) >> 6,
                offset: (instruction & 0b111111) as i16,
            })
        }
        0b1111 => {
            Some(Opcode::TRAP {
                trap_vec: instruction & 0b1111_1111 // This is 0-extended, not sign-extended
            })
        },
        0b1101 => Some(Opcode::RESERVED),
        _ => {
            println!("Unknown instruction {op}");
            None
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
    while pc < 0x3000 + 20 { // Arbitrary limit to debug for now
        let op = decode_instruction(memory[pc]);
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
