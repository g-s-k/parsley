use std::fmt::{Error, Write};

use super::Context;

const PREALLOC_BUFFER: usize = 199;

impl Context {
    /// Start capturing printed content in a buffer.
    pub fn capture(&mut self) {
        self.out = Some(String::with_capacity(PREALLOC_BUFFER));
    }

    /// Capture `display` and `write` statement output in a buffer.
    #[must_use]
    pub fn capturing(mut self) -> Self {
        self.capture();
        self
    }

    /// Get the captured side-effect output.
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
