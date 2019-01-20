use rustyline::error::ReadlineError;
use rustyline::Editor;

use parsley::prelude::*;

const LIB_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPL_WELCOME_BORDER: &str =
    "'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()'()";
const REPL_PROMPT: &str = "> ";
const REPL_HELP: &str = r#"
The following special commands are available:
.help                display this message
.clear               clear the global scope
.exit OR C-c OR C-d  end interactive session
"#;
const REPL_EXIT_MSG: &str = "\nLeaving PARSLEY.\n";

pub fn repl(ctx: &mut Context) -> Result<String, ReadlineError> {
    ctx.push();

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
                rl.add_history_entry(line.as_ref());
                // check for empty line/special commands
                match line.trim() {
                    "" => continue,
                    ".exit" => break Ok(REPL_EXIT_MSG.to_string()),
                    ".clear" => {
                        rl.clear_history();
                        ctx.pop();
                        ctx.push();
                    }
                    ".help" => {
                        println!("{}", REPL_HELP);
                    }
                    other => match ctx.run(other) {
                        Ok(result) => {
                            let res = format!("{}", result);
                            if !res.is_empty() {
                                println!("{}", res);
                            }
                        }
                        Err(error) => println!("{}", error),
                    },
                }
            }
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                break Ok(REPL_EXIT_MSG.to_string());
            }
            Err(error) => break Err(error),
        }
    }
}
