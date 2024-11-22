#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::upper_case_acronyms)]

mod opcodes;
mod terminal;
mod util;
mod vm;

use crate::vm::VM;
use std::fs;

fn main() {
    let data: Vec<u8> = fs::read("2048.obj").expect("Failed to load file");
    let mut vm = VM::new(&data);
    vm.run();
    terminal::restore_terminal_settings();
}
