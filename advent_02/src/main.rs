extern crate clap;

use clap::{App, Arg};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;

fn main() {
    let matches = App::new("Advent_02")
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
    part_two(&contents);
}

fn part_one(contents: &str) {
    println!("Part One.");

    let mut two = 0_u32;
    let mut three = 0_u32;

    for line in contents.lines() {
        let mut char_counts = HashMap::new();
        for character in line.chars() {
            *char_counts.entry(character).or_insert(0) += 1;
        }
        if char_counts.iter().any(|(_, v)| *v == 2) {
            two += 1;
        }
        if char_counts.iter().any(|(_, v)| *v == 3) {
            three += 1;
        }
    }

    println!("  Twos: {}", two);
    println!("  Threes: {}", three);
}

fn part_two(contents: &str) {
    println!("Part Two.");

    // Contains all the single-character wildcard permutations of each hash.
    let mut permutations = HashMap::new();

    for line in contents.lines() {
        let len = line.len();
        for i in 0..len {
            let permutation = line[0..i].to_owned() + "*" + &line[i + 1..len];
            match permutations.entry(permutation.clone()) {
                Entry::Occupied(o) => {
                    println!("  Dup: {}, {}", &permutation, line);
                    println!("       {}, {}", o.key(), o.get());
                    return;
                }
                Entry::Vacant(v) => v.insert(line),
            };
        }
    }
}
