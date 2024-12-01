extern crate clap;

use clap::{App, Arg};
use std::collections::HashSet;
use std::fs;

fn main() {
    let matches = App::new("Advent_01")
        .version("0.1.0")
        .arg(
            Arg::with_name("input_file")
                .required(true)
                .takes_value(true)
                .index(1),
        )
        .get_matches();
    let filename = matches.value_of("input_file").unwrap();
    println!("Parsing input file: {}", filename);

    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    // Part one
    part_one(&contents);

    // Part two
    part_two(&contents)
}

fn part_one(contents: &str) {
    println!("Part One.");

    let mut total: i32 = 0;
    for line in contents.lines() {
        let val: i32 = line.parse().unwrap();
        total += val;
    }

    println!("  Total: {}", total);
}

fn part_two(contents: &str) {
    println!("Part Two.");

    let mut seen = HashSet::new();
    let mut total: i32 = 0;
    loop {
        for line in contents.lines() {
            let val: i32 = line.parse().unwrap();
            total += val;
            if seen.contains(&total) {
                println!("  First see repeat value: {}", total);
                return;
            }
            seen.insert(total);
        }
    }
}
