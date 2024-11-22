#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::upper_case_acronyms)]

use termios::*;

mod opcodes;
mod util;
mod vm;

use crate::vm::VM;
use std::fs;

fn main() {
    // Terminal stuff
    let stdin = 0;
    let termios = termios::Termios::from_fd(stdin).unwrap();
    let mut new_termios = termios.clone();
    new_termios.c_iflag &= IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON;
    new_termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
    tcsetattr(stdin, TCSANOW, &mut new_termios).unwrap();

    let data: Vec<u8> = fs::read("2048.obj").expect("Failed to load file");
    let mut vm = VM::new(&data);
    vm.run();

    // Restore terminal to default settings
    tcsetattr(stdin, TCSANOW, &termios).unwrap();
}
