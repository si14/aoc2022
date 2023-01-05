use std::ops::RangeInclusive;
use std::str::FromStr;

use color_eyre::eyre::{eyre, WrapErr};

use crate::daylib::Day;

type Range = RangeInclusive<usize>;

#[derive(Debug)]
struct ElfPair(Range, Range);

fn parse_range(s: &str) -> color_eyre::Result<Range> {
    let mut iter = s.split('-');
    let (Some(from_s), Some(to_s), None) = (iter.next(), iter.next(), iter.next()) else {
        return Err(eyre!("malformed SectionRange: {s:?}"));
    };
    let from = from_s
        .parse()
        .wrap_err_with(|| format!("can't parse {from_s:?} into int"))?;
    let to = to_s
        .parse()
        .wrap_err_with(|| format!("can't parse {to_s:?} into int"))?;
    Ok(from..=to)
}

impl FromStr for ElfPair {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split(',');
        let (Some(a_s), Some(b_s), None) = (iter.next(), iter.next(), iter.next()) else {
            return Err(eyre!("malformed ElfPair: {s:?}"))
        };

        Ok(ElfPair(
            parse_range(a_s).wrap_err_with(|| format!("can't parse {a_s:?} into a range"))?,
            parse_range(b_s).wrap_err_with(|| format!("can't parse {b_s:?} into a range"))?,
        ))
    }
}

trait InclusiveRangeExt {
    fn contains_range(&self, other: &Range) -> bool;
}

impl InclusiveRangeExt for Range {
    fn contains_range(&self, other: &Range) -> bool {
        self.contains(other.start()) && self.contains(other.end())
    }
}

#[test]
fn test_is_included_in() {
    fn range(start: usize, end: usize) -> Range {
        start..=end
    }

    // positive
    assert!(range(0, 4).contains_range(&range(1, 3)));
    assert!(range(0, 4).contains_range(&range(0, 3)));
    assert!(range(0, 4).contains_range(&range(1, 4)));
    assert!(range(0, 4).contains_range(&range(0, 4)));

    // negative
    assert!(!range(1, 4).contains_range(&range(0, 3)));
    assert!(!range(0, 4).contains_range(&range(1, 5)));
}

fn solve1(input: &str) -> color_eyre::Result<String> {
    let mut n: usize = 0;
    for pair in input.lines().map(ElfPair::from_str) {
        let ElfPair(a, b) = pair?;
        if a.contains_range(&b) || b.contains_range(&a) {
            n += 1;
        }
    }
    Ok(n.to_string())
}

fn is_overlapping(a: &Range, b: &Range) -> bool {
    a.contains(b.start()) || a.contains(b.end()) || b.contains(a.start())
}

#[test]
fn test_is_overlapping() {
    fn range(start: usize, end: usize) -> Range {
        start..=end
    }

    // positive
    assert!(is_overlapping(&range(0, 4), &range(0, 4)));
    assert!(is_overlapping(&range(1, 4), &range(0, 2)));
    assert!(is_overlapping(&range(0, 3), &range(1, 4)));
    assert!(is_overlapping(&range(0, 2), &range(2, 4)));

    // negative
    assert!(!is_overlapping(&range(0, 1), &range(2, 4)));
}

fn solve2(input: &str) -> color_eyre::Result<String> {
    let elf_pairs = input
        .lines()
        .map(ElfPair::from_str)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(elf_pairs
        .iter()
        .filter(|pair| is_overlapping(&pair.0, &pair.1))
        .count()
        .to_string())
}

pub(crate) const DAY: Day = Day {
    number: 4,
    part1: solve1,
    part2: solve2,
};
