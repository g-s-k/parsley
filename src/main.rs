#![feature(const_vec_new)]

#[macro_use]
extern crate failure_derive;
extern crate quicli;
extern crate structopt;

use quicli::prelude::*;
use structopt::StructOpt;

mod lisp;

#[derive(Debug, StructOpt)]
struct Cli {
    file: String,
    #[structopt(flatten)]
    verbosity: Verbosity,
}

fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger("rsch")?;

    let mut base_context = lisp::Context::new();

    info!("Reading source from {}", args.file);
    let code = read_file(&args.file)?;

    info!("Parsing source code.");
    match code.parse::<lisp::SExp>() {
        Ok(tree) => {
            info!("Evaluating.");
            println!("{}", tree.eval(&mut base_context).unwrap());
        }
        Err(error) => error!("{}", error),
    };

    Ok(())
}

