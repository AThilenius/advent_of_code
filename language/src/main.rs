#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate cached;
extern crate clap;
extern crate regex;

#[macro_use]
mod parser;
mod grammar;

use crate::grammar::*;
use crate::parser::*;
use clap::{App, Arg};
use std::fs;

// productions! {

//   program -> Expression {
//     { expr:[expression] _eoi:[end_of_input] } => expr,
//   }

//   // Expressions
//   expression -> Expression {
//     { s:[sum_expression] } => s,
//     { literal:[literal_value] } => Expression::LiteralExpr(literal),
//   }

//   literal_value -> LiteralValue {
//     // { v:[r"[0-9]+\.[0-9]+"] } => LiteralValue::Float64(v.parse::<f64>().unwrap()),
//     { v:[r"[0-9]+"] } => LiteralValue::Int64(v.parse::<i64>().unwrap()),
//   }

//   sum_expression -> Expression {
//     { l:[product_expression] _plus:[r"\+"] r:[sum_expression] }
//         => Expression::BinaryExpr(Box::new(l), BinaryOp::Plus, Box::new(r)),
//     // { l:[product_expression] _:[r"-"] r:[product_expression] }
//     //     => Expression::BinaryExpr(Box::new(l), BinaryOp::Minus, Box::new(r)),
//     { p:[product_expression] } => p,
//   }

//   product_expression -> Expression {
//     { l:[binary_terminal] _star:[r"\*"] r:[product_expression] }
//         => Expression::BinaryExpr(Box::new(l), BinaryOp::Star, Box::new(r)),
//     // { l:[arithmetic_value] _:[r"/"] r:[arithmetic_value] }
//     //     => Expression::BinaryExpr(Box::new(l), BinaryOp::Slash, Box::new(r)),
//     { a:[binary_terminal] } => a,
//   }

//   binary_terminal -> Expression {
//     // { _:[r"\("] expr:[arithmetic_expression] _:[r"\)"] } => expr,
//     { v:[literal_value] } => Expression::LiteralExpr(v),
//   }

// }

// productions! {
//   test_production -> Expression {
//     { expr:[unary_expression] _eoi:[end_of_input] } => expr,
//   }
// }

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

  let mut parser = Parser::new();
  if let Some(res) = parser.parse_or_log_errors(program, &contents) {
    println!("{:#?}", res);
  }

  // let test_source = r"1 + 1 * 1 + 1";
  // let res_opt = parser.parse_or_log_errors(test_production, &test_source);
  // println!("Final results:");
  // if let Some(res) = res_opt {
  //   println!("{:#?}", res);
  // }
}
