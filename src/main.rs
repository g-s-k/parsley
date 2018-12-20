#![feature(const_vec_new)]

#[macro_use]
extern crate failure_derive;
extern crate quicli;
extern crate structopt;

use std::io::{self, Read};
use std::path::PathBuf;

use quicli::prelude::*;
use structopt::StructOpt;

mod lisp;
use self::lisp::{Context, SExp};

#[derive(Debug, StructOpt)]
struct Cli {
    file: Option<PathBuf>,
    #[structopt(flatten)]
    verbosity: Verbosity,
}

fn main() -> CliResult {
    let args = Cli::from_args();
    args.verbosity.setup_env_logger("rsch")?;

    info!("Creating base namespace.");
    let mut base_context = Context::base();


    let code = if let Some(f_name) = args.file {
        info!("Reading source from {:?}", f_name);
        read_file(&f_name)?
    } else {
        let mut code_buffer = String::new();
        io::stdin().read_to_string(&mut code_buffer)?;
        code_buffer
    };

    info!("Parsing source code.");
    match code.parse::<SExp>() {
        Ok(tree) => {
            info!("Evaluating.");
            println!("{}", tree.eval(&mut base_context).unwrap());
        }
        Err(error) => error!("{}", error),
    };

    Ok(())
}
