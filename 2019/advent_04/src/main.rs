#[macro_use]
extern crate lalrpop_util;
extern crate clap;

lalrpop_mod!(pub guard_events);

mod ast;

use clap::{App, Arg};
use std::collections::HashMap;
use std::fs;

fn main() {
    let matches = App::new("Advent_04")
        .arg(
            Arg::with_name("input_file")
                .required(true)
                .takes_value(true),
        )
        .get_matches();
    let filename = matches.value_of("input_file").unwrap();
    let contents = fs::read_to_string(filename).expect("Cannot read file");

    // Part one
    part_one(&contents);
}

fn part_one(contents: &str) {
    println!("Part One.");

    // Parse
    // let mut expr = guard_events::FileParser::new().parse(contents).unwrap();

    // // Accumulate asleep min spectrum for all guards
    // let mut asleep_spectrum = HashMap::new();

    // // Tracked per
    // let mut current_guard_id = -1;

    // for event in expr {
    //     match event {
    //         GuardEvent::Begin(timestamp, id) => current_guard_id = id,
    //         GuardEvent::Sleep(timestamp) =>
    //     }
    // }
}
