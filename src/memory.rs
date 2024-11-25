use crate::util::join_u8;
use thiserror::Error;

const MEMORY_SIZE: usize = 2_usize.pow(16);

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Memory write at {0:#x} out of bounds")]
    OutOfBoundsWrite(usize),
    #[error("Memory read at {0:#x} out of bounds")]
    OutOfBoundsRead(usize),
}

pub(crate) struct Memory {
    data: [u16; MEMORY_SIZE],
}

impl Memory {
    pub(crate) fn new() -> Self {
        Self {
            data: [0u16; MEMORY_SIZE],
        }
    }

    pub(crate) fn write(&mut self, addr: usize, val: u16) {
        let oldval = self
            .data
            .get_mut(addr)
            .unwrap_or_else(|| panic!("{}", MemoryError::OutOfBoundsWrite(addr)));
        *oldval = val;
    }

    pub(crate) fn read(&self, addr: usize) -> u16 {
        self.data
            .get(addr)
            .copied()
            .unwrap_or_else(|| panic!("{}", MemoryError::OutOfBoundsRead(addr)))
    }

    pub(crate) fn load_bulk(&mut self, buff: &[u8]) {
        let origin = join_u8(buff[0], buff[1]) as usize;
        eprintln!("Loading data at origin {:#x}", origin);
        let mut mem_i: usize = origin;
        let mut buff_i: usize = 2; // Skip the origin
        while mem_i < MEMORY_SIZE - 1 && buff_i < buff.len() {
            let word = join_u8(buff[buff_i], buff[buff_i + 1]);
            self.write(mem_i, word);
            mem_i += 1;
            buff_i += 2;
        }
    }
}
