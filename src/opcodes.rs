use crate::util::{sign_ext_imm11, sign_ext_imm5, sign_ext_imm6, sign_ext_imm9};
use crate::vm::TrapCode;

#[derive(Debug, PartialEq, Eq)]
pub enum Argument {
    Reg(usize),
    Immediate(i16),
}

#[derive(Debug, PartialEq, Eq)]
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
                    let imm5 = sign_ext_imm5(instruction);
                    Ok(Opcode::ADD {
                        dr: dr.into(),
                        sr1: sr1.into(),
                        sr2: Argument::Immediate(imm5),
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
                    let imm5 = sign_ext_imm5(instruction);
                    Ok(Opcode::AND {
                        dr: dr.into(),
                        sr1: sr1.into(),
                        sr2: Argument::Immediate(imm5),
                    })
                }
            }
            0b0000 => {
                // BR  Conditional Branch
                let offset = sign_ext_imm9(instruction);
                Ok(Opcode::BR {
                    n: instruction & (1 << 11) != 0,
                    z: instruction & (1 << 10) != 0,
                    p: instruction & (1 << 9) != 0,
                    offset,
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
                    let offset = sign_ext_imm11(instruction);
                    Ok(Opcode::JSR { offset })
                } else {
                    Ok(Opcode::JSRR {
                        base_r: ((instruction & 0b111_000000) >> 6) as usize,
                    })
                }
            }
            0b0010 => {
                // LD
                let offset = sign_ext_imm9(instruction);
                Ok(Opcode::LD {
                    dr: ((instruction & 0b111_0_00000000) >> 9) as usize,
                    offset,
                })
            }
            0b1010 => {
                // LDI , decode same as LD
                let offset = sign_ext_imm9(instruction);
                Ok(Opcode::LDI {
                    dr: ((instruction & 0b111_0_00000000) >> 9) as usize,
                    offset,
                })
            }
            0b0110 => {
                // LDR
                let offset = sign_ext_imm6(instruction);
                Ok(Opcode::LDR {
                    dr: ((instruction & 0b111_000_000000) >> 9) as usize,
                    base_r: ((instruction & 0b000_111_000000) >> 6) as usize,
                    offset,
                })
            }
            0b1110 => {
                // LEA
                let offset = sign_ext_imm9(instruction);
                Ok(Opcode::LEA {
                    dr: ((instruction & 0b111_000000000) >> 9) as usize,
                    offset,
                })
            }
            0b1001 => {
                // NOT
                Ok(Opcode::NOT {
                    dr: ((instruction & 0b111_000_0_00000) >> 9) as usize,
                    sr: ((instruction & 0b000_111_0_00000) >> 6) as usize,
                })
            }
            0b1000 => Ok(Opcode::RTI),
            0b0011 => {
                // ST
                let offset = sign_ext_imm9(instruction);
                Ok(Opcode::ST {
                    sr: ((instruction & 0b111_000000000) >> 9) as usize,
                    offset,
                })
            }
            0b1011 => {
                // STI
                let offset = sign_ext_imm9(instruction);
                Ok(Opcode::STI {
                    sr: ((instruction & 0b111_000000000) >> 9) as usize,
                    offset,
                })
            }
            0b0111 => {
                // STR
                let offset = sign_ext_imm6(instruction);
                Ok(Opcode::STR {
                    sr: ((instruction & 0b111_000_000000) >> 9) as usize,
                    base_r: ((instruction & 0b000_111_000000) >> 6) as usize,
                    offset,
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

#[cfg(test)]
mod tests {
    use crate::opcodes::{Argument, Opcode};
    use crate::vm::TrapCode;

    #[test]
    fn decode_add_reg() {
        // DR=010=2 , SR1=011=3  SR2=001=1
        let ins = 0b0001_010_011_0_00_001;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::ADD {
                dr: 2,
                sr1: 3,
                sr2: Argument::Reg(1)
            }
        );
    }

    #[test]
    fn decode_add_imm5_pos() {
        // DR=000=0 , SR1=100=4  imm5=01001=9
        let ins = 0b0001_000_100_1_01001;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::ADD {
                dr: 0,
                sr1: 4,
                sr2: Argument::Immediate(9)
            }
        );
    }

    #[test]
    fn decode_add_imm5_neg() {
        // DR=000=0 , SR1=100=4  imm5=11011=-5
        let ins = 0b0001_000_100_1_11011;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::ADD {
                dr: 0,
                sr1: 4,
                sr2: Argument::Immediate(-5)
            }
        );
    }

    #[test]
    fn decode_and_reg() {
        // DR=010=2 , SR1=011=3  SR2=001=1
        let ins = 0b0101_010_011_0_00_001;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::AND {
                dr: 2,
                sr1: 3,
                sr2: Argument::Reg(1)
            }
        );
    }

    #[test]
    fn decode_and_imm5_pos() {
        // DR=000=0 , SR1=100=4  imm5=01001=9
        let ins = 0b0101_000_100_1_01001;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::AND {
                dr: 0,
                sr1: 4,
                sr2: Argument::Immediate(9)
            }
        );
    }

    #[test]
    fn decode_and_imm5_neg() {
        // DR=000=0 , SR1=100=4  imm5=10111=-9
        let ins = 0b0101_000_100_1_10111;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::AND {
                dr: 0,
                sr1: 4,
                sr2: Argument::Immediate(-9)
            }
        );
    }

    #[test]
    fn decode_br_pos() {
        let ins = 0b0000_101_000011000;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::BR {
                n: true,
                z: false,
                p: true,
                offset: 24
            }
        );
    }

    #[test]
    fn decode_br_neg() {
        let ins = 0b0000_010_111100000;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::BR {
                n: false,
                z: true,
                p: false,
                offset: -32
            }
        );
    }

    #[test]
    fn decode_br_all_zeroes() {
        let ins = 0b0000_000_000000000;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::BR {
                n: false,
                z: false,
                p: false,
                offset: 0
            }
        );
    }

    #[test]
    fn decode_jmp() {
        let ins = 0b1100_000_011_000000;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(op, Opcode::JMP { base_r: 3 });
    }

    #[test]
    fn decode_ret() {
        let ins = 0b1100_000_111_000000;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(op, Opcode::RET);
    }

    #[test]
    fn decode_jsr_pos() {
        let ins = 0b0100_1_00001110011;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(op, Opcode::JSR { offset: 115 });
    }

    #[test]
    fn decode_jsr_neg() {
        let ins = 0b0100_1_11110010011;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(op, Opcode::JSR { offset: -109 });
    }

    #[test]
    fn decode_jsrr() {
        let ins = 0b0100_0_00_101_000000;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(op, Opcode::JSRR { base_r: 5 });
    }

    #[test]
    fn decode_ld_pos() {
        let ins = 0b0010_101_001000110;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(op, Opcode::LD { dr: 5, offset: 70 });
    }

    #[test]
    fn decode_ld_neg() {
        let ins = 0b0010_101_100000001;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::LD {
                dr: 5,
                offset: -255
            }
        );
    }

    #[test]
    fn decode_ldi_pos() {
        let ins = 0b1010_101_001000110;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(op, Opcode::LDI { dr: 5, offset: 70 });
    }

    #[test]
    fn decode_ldi_neg() {
        let ins = 0b1010_101_100000001;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::LDI {
                dr: 5,
                offset: -255
            }
        );
    }

    #[test]
    fn decode_ldr_pos() {
        let ins = 0b0110_101_001_000111;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::LDR {
                dr: 5,
                base_r: 1,
                offset: 7
            }
        );
    }

    #[test]
    fn decode_ldr_neg() {
        let ins = 0b0110_101_010_100010;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::LDR {
                dr: 5,
                base_r: 2,
                offset: -30
            }
        );
    }

    #[test]
    fn decode_lea_pos() {
        let ins = 0b1110_101_001000110;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(op, Opcode::LEA { dr: 5, offset: 70 });
    }

    #[test]
    fn decode_lea_neg() {
        let ins = 0b1110_101_100000001;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::LEA {
                dr: 5,
                offset: -255
            }
        );
    }

    #[test]
    fn decode_not() {
        let ins = 0b1001_101_001_1_11111;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(op, Opcode::NOT { dr: 5, sr: 1 });
    }

    #[test]
    fn decode_rti() {
        let ins = 0b1000_000000000000;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(op, Opcode::RTI);
    }

    #[test]
    fn decode_st_pos() {
        let ins = 0b0011_101_001000110;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(op, Opcode::ST { sr: 5, offset: 70 });
    }

    #[test]
    fn decode_st_neg() {
        let ins = 0b0011_101_100000001;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::ST {
                sr: 5,
                offset: -255
            }
        );
    }

    #[test]
    fn decode_sti_pos() {
        let ins = 0b1011_101_001000110;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(op, Opcode::STI { sr: 5, offset: 70 });
    }

    #[test]
    fn decode_sti_neg() {
        let ins = 0b1011_101_100000001;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::STI {
                sr: 5,
                offset: -255
            }
        );
    }

    #[test]
    fn decode_str_pos() {
        let ins = 0b0111_101_010_010100;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::STR {
                sr: 5,
                base_r: 2,
                offset: 20
            }
        );
    }

    #[test]
    fn decode_str_neg() {
        let ins = 0b0111_101_010_100010;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::STR {
                sr: 5,
                base_r: 2,
                offset: -30
            }
        );
    }

    #[test]
    fn decode_trap() {
        let ins = 0b1111_0000_0010_0000;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::TRAP {
                trap_code: TrapCode::Getc
            }
        );

        let ins = 0b1111_0000_0010_0001;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::TRAP {
                trap_code: TrapCode::Out
            }
        );

        let ins = 0b1111_0000_0010_0010;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::TRAP {
                trap_code: TrapCode::Puts
            }
        );

        let ins = 0b1111_0000_0010_0011;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::TRAP {
                trap_code: TrapCode::In
            }
        );

        let ins = 0b1111_0000_0010_0100;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::TRAP {
                trap_code: TrapCode::Putsp
            }
        );

        let ins = 0b1111_0000_0010_0101;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(
            op,
            Opcode::TRAP {
                trap_code: TrapCode::Halt
            }
        );
    }

    #[test]
    #[should_panic]
    fn decode_trap_invalid() {
        let ins = 0b1111_0000_0010_0111;
        Opcode::try_from(ins).unwrap();
    }

    #[test]
    fn decode_reserved() {
        let ins = 0b1101_0000_0010_0111;
        let op = Opcode::try_from(ins).unwrap();
        assert_eq!(op, Opcode::RESERVED);
    }
}
