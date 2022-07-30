use rustyline::error::ReadlineError;
use rustyline::Editor;

use parsley::Context;

const NULL: &str = "'()";
const REPL_PROMPT: &str = "> ";
const REPL_WELCOME_MSG: &str = concat!("Welcome to PARSLEY v", env!("CARGO_PKG_VERSION"), ".");
const REPL_EXIT_MSG: &str = "\nLeaving PARSLEY.\n";

pub fn repl(ctx: &mut Context) -> Result<String, ReadlineError> {
    print!(
        "\n{border}\n{side}{line_1:^72}{side}\n{side}{line_2:^72}{side}\n{border}\n\n",
        border = NULL.repeat(26),
        side = NULL,
        line_1 = REPL_WELCOME_MSG,
        line_2 = "Enter `.help` to list special commands."
    );

    let mut rl = Editor::<()>::new()?;

    loop {
        match rl.readline(REPL_PROMPT) {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                // check for empty line/special commands
                match line.trim() {
                    "" => continue,
                    ".exit" => break Ok(REPL_EXIT_MSG.to_string()),
                    ".clear" => {
                        rl.clear_history();
                        ctx.pop();
                    }
                    ".help" => {
                        print!("\n{}\n", include_str!("help.txt"));
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
