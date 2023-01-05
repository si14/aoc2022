use std::collections::BinaryHeap;
use std::iter;

use crate::daylib::Day;

#[derive(Debug)]
struct Elf {
    calories: Vec<u32>,
}

fn parse(input: &str) -> Vec<Elf> {
    input
        .split("\n\n")
        .map(str::trim)
        .map(|x| {
            x.split('\n')
                .map(str::parse)
                .map(Result::unwrap)
                .collect::<Vec<_>>()
        })
        .map(|calories| Elf { calories })
        .collect::<Vec<_>>()
}

#[allow(clippy::unnecessary_wraps)]
fn solve1(input: &str) -> color_eyre::Result<String> {
    let elves = parse(input);

    Ok(elves
        .iter()
        .map(|elf| elf.calories.iter().copied().sum::<u32>())
        .max()
        .unwrap()
        .to_string())
}

#[allow(clippy::unnecessary_wraps)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    let elves = parse(input);

    let mut sorted = elves
        .iter()
        .map(|elf| elf.calories.iter().copied().sum::<u32>())
        .collect::<BinaryHeap<u32>>();

    Ok(iter::from_fn(|| sorted.pop())
        .take(3)
        .sum::<u32>()
        .to_string())
}

pub(crate) const DAY: Day = Day {
    number: 1,
    part1: solve1,
    part2: solve2,
};
