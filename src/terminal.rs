use std::io;
use std::io::{StdinLock, StdoutLock, Write};

pub struct Terminal<'a> {
    stdout: StdoutLock<'a>,
    stdin: StdinLock<'a>,
}

impl Terminal<'_> {
    pub(crate) fn new() -> Self {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        let stdin = io::stdin();
        let stdin = stdin.lock();
        Self { stdout, stdin }
    }
    pub(crate) fn clear(&mut self) {
        write!(self.stdout, "{}", termion::clear::All).unwrap();
    }

    pub(crate) fn out(&mut self, c: u8) {
        self.stdout.write(&[c]).unwrap();
    }
}
