use std::fs;

const MEMORY_SIZE: usize = 2_usize.pow(16);

fn le_to_be(hi: u8, lo: u8) -> u16 {
    let hi = hi as u16;
    let lo = lo as u16;
    (lo << 8) | hi
}

fn read_data_into_memory(data: &[u8], memory: &mut [u16]) {
    let origin = le_to_be(data[0], data[1]) as usize;
    let mut mem_i: usize = origin;
    let mut data_i: usize = 2; // Skip the origin
    while mem_i < MEMORY_SIZE-1 && data_i < data.len() {
        let word = le_to_be(data[data_i], data[data_i+1]);
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
    dbg!(&memory);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_le_to_be() {
        let data: Vec<u8> = vec![0xfe, 0xca];
        let res = le_to_be(data[0], data[1]);
        assert_eq!(0xcafe, res);
    }

    #[test]
    fn test_read_data_in_le() {
        // Memory offset is 0x05dc == 1500
        let data: Vec<u8> = vec![0xdc, 0x05, 0xfe, 0xca, 0xbe, 0xba];
        let mut memory: [u16; MEMORY_SIZE] = [0; MEMORY_SIZE];
        read_data_into_memory(&data, &mut memory);
        for i in 1500_usize..1505_usize {
            println!("{:x?}", memory[i]);
        }
        for i in 0..1500 {
            assert_eq!(memory[i], 0);
        }
        assert_eq!(memory[1500], 0xcafe);
        assert_eq!(memory[1501], 0xbabe);
        for i in 1502..1800 {
            assert_eq!(memory[i], 0);
        }
    }
}
