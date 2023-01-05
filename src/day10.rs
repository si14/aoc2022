use std::str::FromStr;

use color_eyre::eyre::eyre;
use itertools::Itertools;

use crate::daylib::Day;

#[derive(Debug)]
enum Line {
    Noop,
    AddX(isize),
}

impl FromStr for Line {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Line::{AddX, Noop};

        let mut iter = s.split(' ');
        match (iter.next(), iter.next(), iter.next()) {
            (Some("noop"), None, None) => Ok(Noop),
            (Some("addx"), Some(x), None) => Ok(AddX(x.parse().unwrap())),
            other => Err(eyre!("unknown command {other:?}")),
        }
    }
}

fn simulate_lines(lines: &[Line]) -> Vec<isize> {
    use Line::{AddX, Noop};

    lines
        .iter()
        .scan(1, |last_value, line| match line {
            Noop => Some(vec![*last_value]),
            AddX(x) => {
                let new_value = *last_value + x;
                let result = vec![*last_value; 2];
                *last_value = new_value;
                Some(result)
            }
        })
        .flatten()
        .collect()
}

fn solve1(input: &str) -> color_eyre::Result<String> {
    let register_values = simulate_lines(
        &input
            .lines()
            .map(Line::from_str)
            .collect::<Result<Vec<_>, _>>()?,
    );
    println!(
        "{} {} {} {} {} {}",
        register_values[20 - 1],
        register_values[60 - 1],
        register_values[100 - 1],
        register_values[140 - 1],
        register_values[180 - 1],
        register_values[220 - 1]
    );

    Ok((20 * register_values[20 - 1]
        + 60 * register_values[60 - 1]
        + 100 * register_values[100 - 1]
        + 140 * register_values[140 - 1]
        + 180 * register_values[180 - 1]
        + 220 * register_values[220 - 1])
        .to_string())
}

const SCREEN_WIDTH: usize = 40;

#[allow(clippy::cast_possible_wrap)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    let register_values = simulate_lines(
        &input
            .lines()
            .map(Line::from_str)
            .collect::<Result<Vec<_>, _>>()?,
    );

    let screen = Itertools::intersperse(
        register_values
            .iter()
            .enumerate()
            .map(|(crt, register)| {
                if ((crt % SCREEN_WIDTH) as isize - *register).abs() <= 1 {
                    '#'
                } else {
                    '.'
                }
            })
            .chunks(SCREEN_WIDTH)
            .into_iter()
            .map(Iterator::collect::<String>),
        "\n".to_string(),
    )
    .collect::<String>();

    Ok(screen)
}

pub(crate) const DAY: Day = Day {
    number: 10,
    part1: solve1,
    part2: solve2,
};
