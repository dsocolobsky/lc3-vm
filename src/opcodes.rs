use crate::TrapCode;

#[derive(Debug)]
pub enum Argument {
    Reg(usize),
    Immediate(i16),
}

#[derive(Debug)]
pub enum Opcode {
    ADD {
        // Add (dr <- sr1 + sr2/imm5)
        dr: usize,
        sr1: usize,
        sr2: Argument,
    },
    AND {
        /// Bitwise And (dr <- sr1 & sr2/imm5)
        dr: usize,
        sr1: usize,
        sr2: Argument,
    },
    BR {
        // Branch (If n/z/p => pc += offset)
        n: bool,
        z: bool,
        p: bool,
        offset: i16,
    },
    JMP {
        // Jump (pc <- base_r)
        base_r: usize,
    },
    RET, // Return from subroutine (JMP) (pc <- r7)
    JSR {
        // Jump To Subroutine (pc <- pc + offset)
        offset: i16,
    },
    JSRR {
        // Jump to Subroutine (pc <- base_r)
        base_r: usize,
    },
    LD {
        // Load Direct from memory (dr <- mem[pc+offset])
        dr: usize,
        offset: i16,
    },
    LDI {
        // Load Indirect from memory (dr <- mem[mem[pc+offset]])
        dr: usize,
        offset: i16,
    },
    LDR {
        // Load Base+Offset (dr <- mem[base_r + offset])
        dr: usize,
        base_r: usize,
        offset: i16,
    },
    LEA {
        // Load Effective Address (dr <- pc + offset)
        dr: usize,
        offset: i16,
    },
    NOT {
        // Bitwise Not (dr <- ~sr)
        dr: usize,
        sr: usize,
    },
    RTI, // Return From Interrupt
    ST {
        // Store (mem[pc+offset] <- sr)
        sr: usize,
        offset: i16,
    },
    STI {
        // Store Indirect (mem[mem[pc+offset]] <- sr)
        sr: usize,
        offset: i16,
    },
    STR {
        // Store Base+Offset (mem[sr+offset] <- sr)
        sr: usize,
        base_r: usize,
        offset: i16,
    },
    TRAP {
        // Execute Trap
        trap_code: TrapCode,
    },
    RESERVED, // Unused. Throws an Illegal Opcode Exception.
}

impl TryFrom<u16> for Opcode {
    type Error = ();

    fn try_from(instruction: u16) -> Result<Self, Self::Error> {
        let op = (instruction >> 12) as u8; // Highest 4 bits
        match op {
            0b0001 => {
                // ADD
                let dr = (instruction & 0b0000_111_000_0_00_000) >> 9;
                let sr1 = (instruction & 0b0000_000_111_0_00_000) >> 6;
                if instruction & (1 << 5) == 0 {
                    // Use sr2 register as 2nd argument
                    let sr2 = instruction & 0b0000_000_000_0_00_111;
                    Ok(Opcode::ADD {
                        dr: dr.into(),
                        sr1: sr1.into(),
                        sr2: Argument::Reg(sr2.into()),
                    })
                } else {
                    // Use imm5 as 2nd argument
                    let imm5 = instruction & 0b0000_000_000_0_11_111;
                    Ok(Opcode::ADD {
                        dr: dr.into(),
                        sr1: sr1.into(),
                        sr2: Argument::Immediate(imm5 as i16),
                    })
                }
            }
            0b0101 => {
                // AND , this is the same decode case as ADD
                let dr = (instruction & 0b0000_111_000_0_00_000u16) >> 9;
                let sr1 = (instruction & 0b0000_000_111_0_00_000u16) >> 6;
                if instruction & (1 << 5) == 0 {
                    // Use sr2 register as 2nd argument
                    let sr2 = instruction & 0b0000_000_000_0_00_111;
                    Ok(Opcode::AND {
                        dr: dr.into(),
                        sr1: sr1.into(),
                        sr2: Argument::Reg(sr2.into()),
                    })
                } else {
                    // Use imm5 as 2nd argument
                    let imm5 = instruction & 0b0000_000_000_0_11_111;
                    Ok(Opcode::AND {
                        dr: dr.into(),
                        sr1: sr1.into(),
                        sr2: Argument::Immediate(imm5 as i16),
                    })
                }
            }
            0b0000 => {
                // BR  Conditional Branch
                Ok(Opcode::BR {
                    n: instruction & (1 << 11) != 0,
                    z: instruction & (1 << 10) != 0,
                    p: instruction & (1 << 9) != 0,
                    offset: (instruction & 0b1_1111_1111) as i16,
                })
            }
            0b1100 => {
                // JMP/RET
                let base_r = (instruction & 0b0000_000_111_0_00_000) >> 6;
                if base_r == 0b111 {
                    Ok(Opcode::RET)
                } else {
                    Ok(Opcode::JMP {
                        base_r: base_r.into(),
                    })
                }
            }
            0b0100 => {
                // JSR/JSRR
                if instruction & (1 << 11) != 0 {
                    Ok(Opcode::JSR {
                        offset: (instruction & 0b111_11111) as i16,
                    })
                } else {
                    Ok(Opcode::JSRR {
                        base_r: ((instruction & 0b111_000000) >> 6) as usize,
                    })
                }
            }
            0b0010 => {
                // LD
                Ok(Opcode::LD {
                    dr: ((instruction & 0b111_0_00000000) >> 9) as usize,
                    offset: (instruction & 0b1111_1111) as i16,
                })
            }
            0b1010 => {
                // LDI , decode same as LD
                Ok(Opcode::LDI {
                    dr: ((instruction & 0b111_0_00000000) >> 9) as usize,
                    offset: (instruction & 0b1111_1111) as i16,
                })
            }
            0b0110 => {
                // LDR
                Ok(Opcode::LDR {
                    dr: ((instruction & 0b111_000_000000) >> 9) as usize,
                    base_r: ((instruction & 0b000_111_000000) >> 6) as usize,
                    offset: (instruction & 0b000_000_111111) as i16,
                })
            }
            0b1110 => {
                // LEA
                Ok(Opcode::LEA {
                    dr: ((instruction & 0b111_0000_0000) >> 8) as usize,
                    offset: (instruction & 0b000_1111_1111) as i16,
                })
            }
            0b1001 => {
                // NOT
                Ok(Opcode::NOT {
                    dr: ((instruction & 0b111_000_0_00000) >> 9) as usize,
                    sr: ((instruction & 0b111_000_0_00000) >> 9) as usize,
                })
            }
            0b1000 => Ok(Opcode::RTI),
            0b0011 => {
                // ST
                Ok(Opcode::ST {
                    sr: ((instruction & 0b111_0000_0000) >> 8) as usize,
                    offset: (instruction & 0b000_1111_1111) as i16,
                })
            }
            0b1011 => {
                // STI
                Ok(Opcode::STI {
                    sr: ((instruction & 0b111_0000_0000) >> 8) as usize,
                    offset: (instruction & 0b000_1111_1111) as i16,
                })
            }
            0b0111 => {
                // STR
                Ok(Opcode::STR {
                    sr: ((instruction & 0b111_000_000000) >> 9) as usize,
                    base_r: ((instruction & 0b000_111_000000) >> 6) as usize,
                    offset: (instruction & 0b111111) as i16,
                })
            }
            0b1111 => {
                let trap_code_hex = instruction & 0b1111_1111;
                let trap_code = match trap_code_hex {
                    0x20 => TrapCode::Getc,
                    0x21 => TrapCode::Out,
                    0x22 => TrapCode::Puts,
                    0x23 => TrapCode::In,
                    0x24 => TrapCode::Putsp,
                    0x25 => TrapCode::Halt,
                    _ => panic!("Unknown trap code {trap_code_hex} !"),
                };
                Ok(Opcode::TRAP { trap_code })
            }
            0b1101 => Ok(Opcode::RESERVED),
            _ => {
                println!("Unknown instruction {op}");
                Err(())
            }
        }
    }
}
