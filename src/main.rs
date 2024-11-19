use std::fs;

const MEMORY_SIZE: usize = 2_usize.pow(16);

#[derive(Debug)]
enum Opcode {
    BR, // Branch
    ADD, // Add
    LD, // Load
    ST, // Store
    JSR, // Jump Register
    AND, // Bitwise And
    LDR, // Load Register
    STR, // Store Register
    RTI, // Unused
    NOT, // Bitwise Not
    RET, // Return from JSR
    LDI, // Load Indirect
    STI, // Store Indirect
    JMP, // Jump
    RES, // Unused
    LEA, // Load Effective Address
    TRAP // Execute Trap
}

// 1010110101010111
fn decode_instruction(instruction: u16) -> Option<Opcode> {
    let op = (instruction >> 12) as u8; // Highest 4 bits
    match op {
        0b0001 => Some(Opcode::ADD),
        0b0101 => Some(Opcode::AND),
        0b0000 => Some(Opcode::BR),
        0b1100 => Some(Opcode::JMP),
        0b0100 => Some(Opcode::JSR),
        0b0010 => Some(Opcode::LD),
        0b1010 => Some(Opcode::LDI),
        0b0110 => Some(Opcode::LDR),
        0b1110 => Some(Opcode::LEA),
        0b1001 => Some(Opcode::NOT),
        0b1000 => Some(Opcode::RTI),
        0b0011 => Some(Opcode::ST),
        0b1011 => Some(Opcode::STI),
        0b0111 => Some(Opcode::STR),
        0b1111 => Some(Opcode::TRAP),
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
    while mem_i < MEMORY_SIZE-1 && data_i < data.len() {
        let word = join_u8(data[data_i], data[data_i+1]);
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
    while pc < MEMORY_SIZE && pc < (0x3000 + 20){
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
        for i in 0x3000_usize..0x3000+4 {
            println!("{:x?}", memory[i]);
        }
        for i in 0..0x3000 {
            assert_eq!(memory[i], 0);
        }
        assert_eq!(memory[0x3000], 0xcafe);
        assert_eq!(memory[0x3001], 0xbabe);
    }
}
