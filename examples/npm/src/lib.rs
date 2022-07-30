use std::fmt::Write;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Context(parsley::Context);

#[allow(clippy::new_without_default)]
#[wasm_bindgen]
impl Context {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(parsley::Context::base().capturing())
    }

    pub fn run(&mut self, code: &str) -> String {
        // do it
        let evaled = self.0.run(code);

        // get the output
        let mut buf = self.0.get_output().unwrap_or_default();
        self.0.capture();

        // put the results in the string
        let _ = match evaled {
            Ok(exp) => buf.write_fmt(format_args!("{}", exp)),
            Err(error) => buf.write_fmt(format_args!("{}", error)),
        };

        // return
        buf
    }
}
