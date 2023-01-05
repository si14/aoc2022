use std::collections::HashSet;
use std::str::FromStr;

use itertools::Itertools;

use crate::daylib::Day;

#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
struct ItemType {
    code: char,
}

impl ItemType {
    fn priority(self) -> u8 {
        match self.code {
            lower @ 'a'..='z' => lower as u8 - b'a' + 1,
            upper @ 'A'..='Z' => upper as u8 - b'A' + 1 + 26,
            _ => panic!("invalid code {}", self.code),
        }
    }
}

#[test]
fn test_priority() {
    assert_eq!(ItemType { code: 'a' }.priority(), 1);
    assert_eq!(ItemType { code: 'z' }.priority(), 26);
    assert_eq!(ItemType { code: 'A' }.priority(), 27);
    assert_eq!(ItemType { code: 'Z' }.priority(), 52);
}

struct Rucksack(HashSet<ItemType>, HashSet<ItemType>);

impl Rucksack {
    fn full_contents(&self) -> HashSet<ItemType> {
        self.0.union(&self.1).copied().collect()
    }
}

impl FromStr for Rucksack {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (first, second) = s.split_at(s.len() / 2);
        Ok(Rucksack(
            first.chars().map(|code| ItemType { code }).collect(),
            second.chars().map(|code| ItemType { code }).collect(),
        ))
    }
}

#[allow(clippy::unnecessary_wraps)]
fn solve1(input: &str) -> color_eyre::Result<String> {
    Ok(input
        .split('\n')
        .map(Rucksack::from_str)
        .map(Result::unwrap)
        .map(|r| {
            r.0.intersection(&r.1)
                .copied()
                .map(ItemType::priority)
                .map(u16::from)
                .sum::<u16>()
        })
        .sum::<u16>()
        .to_string())
}

#[allow(clippy::unnecessary_wraps)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    Ok(input
        .split('\n')
        .map(Rucksack::from_str)
        .map(Result::unwrap)
        .chunks(3)
        .into_iter()
        .map(|mut chunk| {
            let first = chunk.next().unwrap();
            // chunk doesn't contain first anymore
            chunk.fold(first.full_contents(), |candidates, new_set| {
                candidates
                    .intersection(&new_set.full_contents())
                    .copied()
                    .collect()
            })
        })
        .inspect(|badge_items| {
            assert_eq!(
                badge_items.len(),
                1,
                "only one badge per elf group, got {:?}",
                badge_items
            );
        })
        .map(|badge_items| u16::from(badge_items.iter().next().unwrap().priority()))
        .sum::<u16>()
        .to_string())
}

pub(crate) const DAY: Day = Day {
    number: 3,
    part1: solve1,
    part2: solve2,
};
