#![feature(box_patterns)]
#![deny(missing_docs)]
#![deny(unused_must_use)]
//! An interpreter for a language,
//! This is an experiment.
//!
extern crate clap;
extern crate rand;
#[macro_use]
extern crate lazy_static;

extern crate time;
mod compile;
//mod compile_req;
mod expression;
mod keywords;
mod native;
mod parser;
mod scanner;
pub mod statement;
mod token;
mod typecheck;
mod types;
mod vm;
mod lisaa;

#[allow(unused_imports)]
use std::io::{Read, self};

use clap::{App, Arg};
#[allow(unused_imports)]
use std::fs::File;
use std::process::exit;
use lisaa::Lisaa;
fn main() {
    let matches = App::new("Lisaa")
        .version("0.0.0")
        .author("Pierre Bertin-Johannet")
        .about("Interpreter for the Lisaa lang")
        .arg(
            Arg::with_name("INPUT")
                .index(1)
                .help("the file to run")
                .default_value("test.lisaa")
                .required(true),
        )
        .get_matches();

    let input_file = matches.value_of("INPUT").unwrap();
    let mut stdout = io::stdout();
    exit(match Lisaa::new(input_file.to_owned(), &mut stdout).run() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}
