use std::io::{self, Write};

use quicli::prelude::*;

use parsley::{Context, SExp};

const LIB_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPL_WELCOME_BORDER: &str =
    "'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()";
const REPL_PROMPT: &str = "> ";
const REPL_HELP: &str = r#"
The following special commands are available:
.help     display this message
.clear    clear the global scope
.exit     end interactive session
"#;

pub fn repl(ctx: &mut Context) -> io::Result<usize> {
    info!("Initializing REPL.");

    let mut buffer = String::new();

    print!(
        "\n{0}\n'(){1:^72}'()\n'(){2:^72}'()\n{0}\n\n",
        REPL_WELCOME_BORDER,
        format!("Welcome to PARSLEY v{}.", LIB_VERSION),
        "Enter `.help` to list special commands."
    );

    loop {
        // write prompt and ensure it actually prints
        print!("{}", REPL_PROMPT);
        io::stdout().flush()?;

        match io::stdin().read_line(&mut buffer) {
            Ok(n) => {
                // check for empty line
                if n < 2 {
                    continue;
                }

                // check for special commands
                match buffer.trim() {
                    ".exit" => break Ok(0),
                    ".clear" => {
                        ctx.pop();
                    }
                    ".help" => {
                        println!("{}", REPL_HELP);
                    }
                    other => match other.parse::<SExp>() {
                        Ok(tree) => match tree.eval(ctx) {
                            Ok(result) => println!("{}", result),
                            Err(error) => println!("{}", error),
                        },
                        Err(error) => println!("{}", error),
                    },
                }
            }
            error => break error,
        }
        buffer.clear();
    }
}
