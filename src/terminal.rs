use std::io;
use std::io::{StdinLock, StdoutLock, Write};
use termion::input::{Keys, TermRead};

pub struct Terminal<'a> {
    stdout: StdoutLock<'a>,
    pub stdin: Keys<StdinLock<'a>>,
}

impl Terminal<'_> {
    pub(crate) fn new() -> Self {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        let stdin = io::stdin();
        let stdin = stdin.lock();
        let keys = stdin.keys();
        Self { stdout, stdin: keys }
    }
    pub(crate) fn clear(&mut self) {
        write!(self.stdout, "{}", termion::clear::All).unwrap();
    }

    pub(crate) fn out(&mut self, c: u8) {
        self.stdout.write(&[c]).unwrap();
    }

    pub(crate) fn flush(&mut self) {
        self.stdout.flush().unwrap()
    }
}
