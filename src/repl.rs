use rustyline::{error, Editor};

use parsley::prelude::*;

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

pub fn repl(ctx: &mut Context) -> Result<String, error::ReadlineError> {
    info!("Initializing REPL.");

    print!(
        "\n{0}\n'(){1:^72}'()\n'(){2:^72}'()\n{0}\n\n",
        REPL_WELCOME_BORDER,
        format!("Welcome to PARSLEY v{}.", LIB_VERSION),
        "Enter `.help` to list special commands."
    );

    let mut rl = Editor::<()>::new();

    loop {
        let readline = rl.readline(REPL_PROMPT);

        match readline {
            Ok(line) => {
                // check for empty line/special commands
                match line.trim() {
                    "" => continue,
                    ".exit" => break Ok(line),
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
            Err(error) => break Err(error),
        }
    }
}
