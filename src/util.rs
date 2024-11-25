pub(crate) fn join_u8(hi: u8, lo: u8) -> u16 {
    let hi = hi as u16;
    let lo = lo as u16;
    (hi << 8) | lo
}

pub(crate) fn sign_ext_imm6(instruction: u16) -> i16 {
    let offset = (instruction & 0b11_1111) as i16;
    if offset & 0b10_0000 != 0 {
        offset | !0b11_1111
    } else {
        offset & 0b11_1111
    }
}

pub(crate) fn sign_ext_imm9(instruction: u16) -> i16 {
    let offset = (instruction & 0b1_1111_1111) as i16;
    if offset & 0b1_0000_0000 != 0 {
        offset | !0b1_1111_1111
    } else {
        offset & 0b1_1111_1111
    }
}

pub(crate) fn sign_ext_imm5(instruction: u16) -> i16 {
    let imm5 = (instruction & 0b0000_000_000_0_11_111) as i16;
    if imm5 & 0b1_0000 != 0 {
        imm5 | !0b0001_1111
    } else {
        imm5 & 0b0001_1111
    }
}

pub(crate) fn sign_ext_imm11(instruction: u16) -> i16 {
    let offset = (instruction & 0b111_1111_1111) as i16;
    if offset & 0b100_0000_0000 != 0 {
        offset | !0b111_1111_1111
    } else {
        offset & 0b111_1111_1111
    }
}
