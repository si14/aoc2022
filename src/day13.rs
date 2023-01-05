use std::cmp;

use crate::daylib::Day;
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map, map_res},
    multi::separated_list0,
    sequence::{delimited, separated_pair},
    Finish, IResult,
};

#[derive(Debug, Eq, PartialEq, Clone)]
enum Item {
    Int(u8),
    List(Vec<Item>),
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Item) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        use Item::{Int, List};

        match (self, other) {
            (Int(a), Int(b)) => a.cmp(b),
            (a @ Int(_), b @ List(_)) => List(vec![a.clone()]).cmp(b),
            (a @ List(_), b @ Int(_)) => a.cmp(&List(vec![b.clone()])),
            (List(a), List(b)) => a.cmp(b),
        }
    }
}

impl Item {
    fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            map_res(digit1, |s| str::parse::<u8>(s).map(Item::Int)),
            map(
                delimited(tag("["), separated_list0(tag(","), Item::parse), tag("]")),
                Item::List,
            ),
        ))(i)
    }
}

#[test]
fn test_item_parse() {
    #[cfg(test)]
    use pretty_assertions::assert_eq;
    use Item::{Int, List};

    assert_eq!(
        Item::parse("[1,1,3]"),
        Ok(("", List(vec![Int(1), Int(1), Int(3)])))
    );
    assert_eq!(
        Item::parse("[1,[1],3,[]]"),
        Ok((
            "",
            List(vec![Int(1), List(vec![Int(1)]), Int(3), List(vec![])])
        ))
    );
    assert_eq!(Item::parse("5"), Ok(("", Int(5))));
}

fn parse_pair(i: &str) -> IResult<&str, (Item, Item)> {
    separated_pair(Item::parse, tag("\n"), Item::parse)(i)
}

#[allow(clippy::unnecessary_wraps)]
fn solve1(input: &str) -> color_eyre::Result<String> {
    Ok(input
        .split("\n\n")
        .map(|s| parse_pair(s).finish().unwrap().1)
        .map(|(l, r)| l.cmp(&r))
        .enumerate()
        .filter_map(|(i, ord)| if ord.is_lt() { Some(i + 1) } else { None })
        .sum::<usize>()
        .to_string())
}

#[allow(clippy::unnecessary_wraps)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    let divider_packets = vec![
        Item::parse("[[2]]").unwrap().1,
        Item::parse("[[6]]").unwrap().1,
    ];

    Ok(input
        .split('\n')
        .filter(|s| !s.is_empty())
        .map(|s| Item::parse(s).unwrap().1)
        .chain(divider_packets.clone())
        .sorted()
        .enumerate()
        .filter_map(|(i, p)| {
            if p == divider_packets[0] || p == divider_packets[1] {
                Some(i + 1)
            } else {
                None
            }
        })
        .product::<usize>()
        .to_string())
}

pub(crate) const DAY: Day = Day {
    number: 13,
    part1: solve1,
    part2: solve2,
};
