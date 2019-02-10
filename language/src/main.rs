#[macro_use]
extern crate lalrpop_util;
extern crate clap;
extern crate regex;

mod ast;
mod scope;
mod vm;

use crate::vm::*;
use clap::{App, Arg};
use std::fs;

fn main() {
    let matches = App::new("atc")
        .arg(
            Arg::with_name("input_file")
                .required(true)
                .takes_value(true),
        )
        .get_matches();
    let filename = matches.value_of("input_file").unwrap();
    let contents = fs::read_to_string(filename).expect("Cannot read file");

    let mut vm = VM::new();
    vm.exec(&contents);
}
