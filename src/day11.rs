use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map_res, opt, value},
    multi::separated_list0,
    sequence::{delimited, pair, separated_pair},
    IResult,
};

use crate::daylib::Day;

#[derive(Debug, Clone, Copy)]
enum Term {
    Old,
    Const(i32),
}

impl Term {
    fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            value(Self::Old, tag("old")),
            map_res(
                pair(opt(tag("-")), digit1),
                |(neg, value_s): (Option<&str>, &str)| {
                    let value: i32 = value_s.parse()?;
                    let mult = if neg.is_none() { 1 } else { -1 };
                    Ok::<_, color_eyre::Report>(Self::Const(value * mult))
                },
            ),
        ))(i)
    }
}

#[derive(Debug, Clone, Copy)]
enum Op {
    Mult,
    Add,
}

impl Op {
    fn parse(i: &str) -> IResult<&str, Self> {
        alt((value(Self::Mult, tag("*")), value(Self::Add, tag("+"))))(i)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct Item {
    worry: u64,
}

impl Item {
    fn parse(i: &str) -> IResult<&str, Self> {
        map_res(digit1, |x: &str| {
            Ok::<_, color_eyre::Report>(Self { worry: x.parse()? })
        })(i)
    }
}

#[derive(Debug, Clone)]
struct Monkey {
    id: u32,
    items: Vec<Item>,
    operation: (Op, Term),
    test_divider: u32,
    throw_to_true: usize,
    throw_to_false: usize,
}

impl Monkey {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, id) = map_res(
            delimited(tag("Monkey "), digit1, tag(":\n")),
            str::parse::<u32>,
        )(i)?;
        let (i, starting_items) = delimited(
            tag("  Starting items: "),
            separated_list0(tag(", "), Item::parse),
            tag("\n"),
        )(i)?;
        let (i, operation) = delimited(
            tag("  Operation: new = old "),
            separated_pair(Op::parse, tag(" "), Term::parse),
            tag("\n"),
        )(i)?;
        let (i, test_divider) = map_res(
            delimited(tag("  Test: divisible by "), digit1, tag("\n")),
            str::parse::<u32>,
        )(i)?;
        let (i, throw_to_true) = map_res(
            delimited(tag("    If true: throw to monkey "), digit1, tag("\n")),
            str::parse::<usize>,
        )(i)?;
        let (i, throw_to_false) = map_res(
            delimited(
                tag("    If false: throw to monkey "),
                digit1,
                opt(tag("\n")),
            ),
            str::parse::<usize>,
        )(i)?;

        Ok((
            i,
            Monkey {
                id,
                items: starting_items,
                operation,
                test_divider,
                throw_to_true,
                throw_to_false,
            },
        ))
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_lossless
    )]
    fn inspect(&self, item: Item, reduce_worry: bool, divider_mult: u32) -> (usize, Item) {
        use Op::{Add, Mult};
        use Term::{Const, Old};

        let mut new_worry = item.worry;

        match self.operation {
            (Mult, Old) => new_worry *= new_worry,
            (Mult, Const(x)) => new_worry *= x as u64,
            (Add, Old) => new_worry += new_worry,
            (Add, Const(x)) => new_worry += x as u64,
        }

        if reduce_worry {
            new_worry /= 3;
        }

        new_worry %= divider_mult as u64;

        let new_item = Item { worry: new_worry };

        if (new_worry as u32) % self.test_divider == 0 {
            (self.throw_to_true, new_item)
        } else {
            (self.throw_to_false, new_item)
        }
    }
}

#[allow(clippy::unnecessary_wraps)]
fn solve(
    input: &str,
    rounds: u32,
    reduce_worry: bool,
    print_trace: bool,
) -> color_eyre::Result<String> {
    let mut state = input
        .split("\n\n")
        .map(|s| Monkey::parse(s).unwrap().1)
        .collect::<Vec<_>>();
    let mut counts = vec![0u64; state.len()];

    let divider_mult = state.iter().map(|m| m.test_divider).product::<u32>();

    for round_i in 0..rounds {
        for monkey_i in 0..state.len() {
            counts[monkey_i] += state[monkey_i].items.len() as u64;
            let changes = state[monkey_i]
                .items
                .iter()
                .copied()
                .map(|it| state[monkey_i].inspect(it, reduce_worry, divider_mult))
                .collect::<Vec<_>>();
            for (throw_to, item) in changes {
                state[throw_to].items.push(item);
            }
            state[monkey_i].items.drain(..);
        }
        if print_trace {
            println!(
                "after round {round_i}:\n{}\n\n",
                Itertools::intersperse(
                    state.iter().map(|m| format!(
                        "Monkey {}: {:?}",
                        m.id,
                        m.items.iter().map(|x| x.worry).collect::<Vec<_>>()
                    )),
                    "\n".to_string()
                )
                .collect::<String>()
            );
        }
    }
    println!("inspections: {counts:?}");

    Ok(format!(
        "monkey business level: {}",
        counts.iter().sorted().rev().take(2).product::<u64>()
    ))
}

fn solve1(input: &str) -> color_eyre::Result<String> {
    solve(input, 20, true, false)
}

fn solve2(input: &str) -> color_eyre::Result<String> {
    solve(input, 10_000, false, false)
}

pub(crate) const DAY: Day = Day {
    number: 11,
    part1: solve1,
    part2: solve2,
};
