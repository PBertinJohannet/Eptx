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
mod compile_req;
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
mod ytp;
use std::io::Read;

use clap::{App, Arg};
use std::fs::File;
use std::process::exit;
use ytp::Ytp;

fn main() {
    let matches = App::new("Lisaa")
        .version("0.0.0")
        .author("Pierre Bertin-Johannet")
        .about("Interpreter for the Lisaa lang")
        .arg(
            Arg::with_name("INPUT")
                .index(1)
                .help("the file to run")
                .default_value("pi.lisaa")
                .required(true),
        )
        .get_matches();

    let input_file = matches.value_of("INPUT").unwrap();

    exit(match run_file(input_file) {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}

/// Runs the given file with the interpreter.
fn run_file(input_file: &str) -> Result<(), String> {
    let mut file = match File::open(input_file) {
        Ok(file) => file,
        Err(e) => {
            return Err(format!(
                "could not open file : {},\
                 error : {} ",
                input_file, e
            ).to_string());
        }
    };

    println!("\nlisaa : Running {}\n\n", input_file);

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(file) => file,
        Err(e) => {
            return Err(format!(
                "could not read file : {},\
                 error : {} ",
                input_file, e
            ).to_string());
        }
    };
    return Ytp::new(contents).run();
}
