#![allow(clippy::unusual_byte_groupings)]
#![allow(clippy::upper_case_acronyms)]

use termios::*;

mod memory;
mod opcodes;
mod util;
mod vm;

use crate::vm::VM;
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    let Some(filename) = args.get(1) else {
        println!("You must provide an obj file");
        std::process::exit(1);
    };

    // Terminal stuff
    let stdin = 0;
    let mut termios = Termios::from_fd(stdin).expect("failed to initialize terminal");
    termios.c_iflag &= IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON;
    termios.c_lflag &= !(ICANON | ECHO); // no echo and canonical mode
    tcsetattr(stdin, TCSANOW, &termios).expect("failed to initialize terminal");

    println!("Loading file {filename}");
    let data: Vec<u8> = fs::read(filename).expect("Failed to load file");
    let mut vm = VM::new(&data);
    vm.run();

    // Restore terminal to default settings
    tcsetattr(stdin, TCSANOW, &termios).expect("failed to close terminal");
}
