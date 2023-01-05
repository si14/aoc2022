extern crate core;

use std::fs;
use std::str::FromStr;

use bpaf::{construct, positional, OptionParser, Parser};
use color_eyre::eyre::{eyre, WrapErr};

use crate::daylib::Day;

mod day01;
mod day02;
mod day03;
mod day04;
mod day05;
mod day06;
mod day07;
mod day08;
mod day09;
mod day10;
mod day11;
mod day12;
mod day13;
mod day14;
mod day15;
mod day16;
mod day17;
mod day18;
mod day19;
mod day20;
mod day21;
mod day22;
mod day23;
mod day24;
mod day25;
mod daylib;
mod shared;

#[derive(Debug, Clone)]
enum DayPart {
    First,
    Second,
}

impl FromStr for DayPart {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> color_eyre::Result<Self> {
        use DayPart::{First, Second};

        match s {
            "1" => Ok(First),
            "2" => Ok(Second),
            other => Err(eyre!("expected 1 or 2, got {}", other)),
        }
    }
}

#[derive(Debug, Clone)]
struct Opts {
    day: u8,
    part: DayPart,
    input_flavour: Option<String>,
}

fn options() -> OptionParser<Opts> {
    let day = positional::<u8>("day").help("Which day it is?");
    let part = positional::<DayPart>("part").help("Which part it is? (1 or 2)");
    let input_flavour = positional("input_flavour")
        .help("Which input file to use (input_FLAVOUR.txt), uses input.txt if not set")
        .optional();

    construct!(Opts {
        day,
        part,
        input_flavour
    })
    .to_options()
    .descr("Advent of Code 2022 solver")
}

const DAYS: [Day; 25] = [
    day01::DAY,
    day02::DAY,
    day03::DAY,
    day04::DAY,
    day05::DAY,
    day06::DAY,
    day07::DAY,
    day08::DAY,
    day09::DAY,
    day10::DAY,
    day11::DAY,
    day12::DAY,
    day13::DAY,
    day14::DAY,
    day15::DAY,
    day16::DAY,
    day17::DAY,
    day18::DAY,
    day19::DAY,
    day20::DAY,
    day21::DAY,
    day22::DAY,
    day23::DAY,
    day24::DAY,
    day25::DAY,
];

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let opts = options().run();

    let day_struct = DAYS
        .iter()
        .find(|d| d.number == opts.day)
        .unwrap_or_else(|| panic!("unexpected day {}", opts.day));

    let solver = match opts.part {
        DayPart::First => day_struct.part1,
        DayPart::Second => day_struct.part2,
    };

    let input_path = if let Some(flavour) = opts.input_flavour {
        format!("data/day{}/input_{flavour}.txt", opts.day)
    } else {
        format!("data/day{}/input.txt", opts.day)
    };

    let input =
        fs::read_to_string(&input_path).wrap_err(format!("input error at {}", &input_path))?;
    println!("result:\n{}", solver(&input).wrap_err("solver error")?);

    Ok(())
}

#[test]
fn check_options() {
    options().check_invariants(true);
}
