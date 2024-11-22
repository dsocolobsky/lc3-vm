use libc::c_int;
use libc::STDIN_FILENO;
use termios::*;

pub fn restore_terminal_settings() {
    let mut term: Termios = Termios::from_fd(STDIN_FILENO).unwrap();
    //turn on canonical mode and echo mode
    term.c_lflag |= ICANON | ECHO;
    tcsetattr(STDIN_FILENO, TCSANOW, &term).unwrap();
}

pub fn turn_off_canonical_and_echo_modes() {
    let mut term: Termios = Termios::from_fd(STDIN_FILENO).unwrap();
    //turn off canonical mode and echo mode
    term.c_lflag &= !(ICANON | ECHO);
    tcsetattr(STDIN_FILENO, TCSANOW, &term).unwrap();
}

extern "C" {
    fn getchar() -> c_int;
}

/// `get_char` calls external (C) function getchar using libc.
pub fn get_char() -> i32 {
    unsafe { getchar() }
}
