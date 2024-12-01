extern crate clap;
extern crate image;
extern crate regex;

#[macro_use]
extern crate lazy_static;

use clap::{App, Arg};
use image::{ImageBuffer, Rgb};
use regex::Regex;
use std::fs;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
struct Claim {
    id: String,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
}

lazy_static! {
    static ref RE: Regex = Regex::new(r"^#(\d+)\s*@\s*(\d+),(\d+):\s*(\d+)x(\d+)$").unwrap();
}

impl FromStr for Claim {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let caps = RE.captures(s).unwrap();

        Ok(Claim {
            id: caps.get(1).unwrap().as_str().to_owned(),
            left: caps.get(2).unwrap().as_str().parse().unwrap(),
            top: caps.get(3).unwrap().as_str().parse().unwrap(),
            width: caps.get(4).unwrap().as_str().parse().unwrap(),
            height: caps.get(5).unwrap().as_str().parse().unwrap(),
        })
    }
}

fn main() {
    let matches = App::new("Advent_03")
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
}

fn part_one(contents: &str) {
    println!("Part One.");

    // Start by parsing into Claim structs
    let claims: Vec<Claim> = contents.lines().map(|line| line.parse().unwrap()).collect();
    println!("  Claims: {}", claims.len());

    let mut image = ImageBuffer::<Rgb<u8>, Vec<u8>>::new(1000, 1000);
    let mut overlap = [[0 as u32; 1000]; 1000];

    for claim in claims.iter() {
        for x in claim.left..claim.left + claim.width {
            for y in claim.top..claim.top + claim.height {
                overlap[x as usize][y as usize] += 1_u32;
            }
        }
    }

    for claim in claims.iter() {
        for x in claim.left..claim.left + claim.width {
            image.get_pixel_mut(x, claim.top).data = [255, 255, 255];
            image.get_pixel_mut(x, claim.top + claim.height - 1).data = [255, 255, 255];
        }
        for y in claim.top..claim.top + claim.height {
            image.get_pixel_mut(claim.left, y).data = [255, 255, 255];
            image.get_pixel_mut(claim.left + claim.width - 1, y).data = [255, 255, 255];
        }
        for x in claim.left..claim.left + claim.width {
            for y in claim.top..claim.top + claim.height {
                if overlap[x as usize][y as usize] > 1 {
                    image.get_pixel_mut(x, y).data = [255, 0, 0];
                }
            }
        }
    }

    let mut overlap_inches = 0;
    for x in 0..1000 {
        for y in 0..1000 {
            if overlap[x as usize][y as usize] > 1 {
                overlap_inches += 1;
            }
        }
    }
    println!("  Overlap Inches: {}", overlap_inches);

    'outer: for claim in claims.iter() {
        for x in claim.left..claim.left + claim.width {
            for y in claim.top..claim.top + claim.height {
                if overlap[x as usize][y as usize] > 1 {
                    continue 'outer;
                }
            }
        }
        println!("  Claim {} didn't overlap", claim.id);
    }

    image.save("output.png").unwrap();
}
