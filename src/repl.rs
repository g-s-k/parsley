use std::io::{self, Write};

use quicli::prelude::*;

use parsley::{self, Context};

const REPL_PROMPT: &str = "> ";

pub fn repl(ctx: &mut Context) -> io::Result<usize> {
    info!("Initializing REPL.");

    println!();
    println!("'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()");
    println!("'()          Welcome to PARSLEY, an interactive Scheme interpreter.        '()");
    println!("'()          You are using version 0.1.0.                                  '()");
    println!("'()          Press <CTRL>-C or enter `.exit` to quit.                      '()");
    println!("'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()");
    println!();

    let mut buffer = String::new();

    io::stdout().flush()?;
    loop {
        // write prompt and ensure it actually prints
        io::stdout().write_all(REPL_PROMPT.as_bytes())?;
        io::stdout().flush()?;

        match io::stdin().read_line(&mut buffer) {
            Ok(n) => {
                // check for empty line
                if n < 2 {
                    continue;
                }

                // check for exit command
                if buffer.trim() == ".exit" {
                    break Ok(0);
                }

                match parsley::parse(&buffer) {
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
