#[macro_use]
extern crate log;

use std::io::{self, Read};
use std::path::PathBuf;

use quicli::prelude::*;
use structopt::StructOpt;

use parsley::{Context, SExp};
mod repl;

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short = "i", long = "interactive")]
    force_interactive: bool,
    #[structopt(short = "s", long = "stdin")]
    read_stdin: bool,
    #[structopt(parse(from_os_str))]
    file: Option<PathBuf>,
    #[structopt(flatten)]
    verbosity: Verbosity,
}

fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger("parsley")?;

    info!("Creating base namespace.");
    let mut base_context = Context::base();

    let code = if let Some(f_name) = args.file {
        info!("Reading source from {:?}", f_name);
        read_file(&f_name)?
    } else if args.read_stdin {
        let mut code_buffer = String::new();
        io::stdin().read_to_string(&mut code_buffer)?;
        code_buffer
    } else {
        String::new()
    };

    if !code.is_empty() {
        info!("Parsing source code.");
        match code.parse::<SExp>() {
            Ok(tree) => {
                info!("Evaluating.");
                println!("{}", tree.eval(&mut base_context).unwrap());
            }
            Err(error) => error!("{}", error),
        };
    }

    if code.is_empty() || args.force_interactive {
        match repl::repl(&mut base_context) {
            Ok(res) => println!("{}", res),
            Err(err) => error!("{}", err),
        }
    }

    Ok(())
}
