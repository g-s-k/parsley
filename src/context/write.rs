use std::fmt::{Error, Write};

use super::Context;

const PREALLOC_BUFFER: usize = 199;

impl Context {
    pub fn capture(&mut self) {
        self.out = Some(String::with_capacity(PREALLOC_BUFFER));
    }

    pub fn capturing(mut self) -> Self {
        self.capture();
        self
    }

    pub fn get_output(&mut self) -> Option<String> {
        self.out.take()
    }
}

impl Write for Context {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        if let Some(ref mut st) = &mut self.out {
            write!(st, "{}", s)
        } else {
            print!("{}", s);
            Ok(())
        }
    }
}
