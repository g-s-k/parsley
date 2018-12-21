use std::io::{self, Write};

use quicli::prelude::*;

use parsley::{Context, SExp};

const REPL_PROMPT: &str = "> ";
const REPL_WELCOME: &str = r#"
'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()
'()          Welcome to PARSLEY, an interactive Scheme interpreter.        '()
'()          You are using version 0.1.0.                                  '()
'()          Press <CTRL>-C or enter `.exit` to quit.                      '()
'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()

"#;

pub fn repl(ctx: &mut Context) -> io::Result<usize> {
    info!("Initializing REPL.");

    let mut buffer = String::new();

    io::stdout().write_all(REPL_WELCOME.as_bytes())?;
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
