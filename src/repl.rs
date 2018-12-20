use std::io::{self, Write};

use quicli::prelude::*;

use crate::lisp::{Context, SExp};

const REPL_PROMPT: &'static str = "> ";

pub fn repl(ctx: &mut Context) -> io::Result<usize> {
    info!("Initializing REPL.");

    println!("\nWelcome to PARSE, an interactive Scheme interpreter.");
    println!("Press <CTRL>-C or enter `.exit` to quit\n");

    let mut buffer = String::new();

    loop {
        // write prompt and ensure it actually prints
        io::stdout().write(REPL_PROMPT.as_bytes())?;
        io::stdout().flush()?;

        match io::stdin().read_line(&mut buffer) {
            Ok(n) => {
                // check for empty line
                if n < 2 {
                    continue;
                }

                // check for exit command
                if buffer.trim() == ".exit" {
                    break Ok(0)
                }

                match buffer.parse::<SExp>() {
                    Ok(tree) => match tree.eval(ctx) {
                        Ok(result) => println!("{}", result),
                        Err(error) => println!("{}", error),
                    },
                    Err(error) => println!("{}", error),
                }
            }
            error => break error,
        }
        buffer.clear();
    }
}
