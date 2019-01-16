use std::fs;
use std::io::{self, Read, Result};
use std::path::PathBuf;

use structopt::StructOpt;

use parsley::prelude::*;
mod repl;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "i", long = "interactive")]
    force_interactive: bool,
    #[structopt(short = "s", long = "stdin")]
    read_stdin: bool,
    #[structopt(parse(from_os_str))]
    file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Cli::from_args();

    let mut base_context = Context::base();

    let code = if let Some(f_name) = args.file {
        fs::read_to_string(&f_name)?
    } else if args.read_stdin {
        let mut code_buffer = String::new();
        io::stdin().read_to_string(&mut code_buffer)?;
        code_buffer
    } else {
        String::new()
    };

    if !code.is_empty() {
        match run_in(&code, &mut base_context) {
            Ok(tree) => {
                println!("{}", tree);
            }
            Err(error) => eprintln!("{}", error),
        };
    }

    if code.is_empty() || args.force_interactive {
        match repl::repl(&mut base_context) {
            Ok(res) => println!("{}", res),
            Err(err) => eprintln!("{}", err),
        }
    }

    Ok(())
}
