#[macro_use]
extern crate lazy_static;

// extern crate clap;
extern crate regex;

// mod ast;
#[macro_use]
mod parser;
mod grammar;
// mod scope;
// mod vm;

// use crate::vm::*;
// use clap::{App, Arg};
// use std::fs;
use crate::parser::*;

productions! {

  ident -> &'a str {
    { name:[r"[a-zA-Z][a-zA-Z0-9_-]*"] } => name
  }

  statement -> String {
    { _:[r"let"] ident:[ident] _:[r"="] expr:[expression] _:[r";"] } => format!("let {} = {};", ident, expr)
  }

  expression -> String {
    { ident:[ident] } => ident.to_owned(),
    { _:[r"\{"] expr:[expression] _:[r"\}"] } => expr,
  }

}

fn main() {
  // let matches = App::new("atc")
  //     .arg(
  //         Arg::with_name("input_file")
  //             .required(true)
  //             .takes_value(true),
  //     )
  //     .get_matches();
  // let filename = matches.value_of("input_file").unwrap();
  // let contents = fs::read_to_string(filename).expect("Cannot read file");

  // let mut vm = VM::new();
  // vm.exec(&contents);

  // let sample = r#"
  //     "Hello, \"world!" "And another!"
  // "#;
  // let parser = Parser::new(sample);
  // let tokens = parser.parse();
  // println!("{:#?}", tokens);

  let source = r"let foo_bar = { baz_bang };";
  let mut parser = Parser::new();
  if let Some(res) = parser.parse_or_log_errors(statement, source) {
    println!("{:#?}", res);
  }
}
