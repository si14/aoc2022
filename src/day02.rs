use std::fmt;
use std::str::FromStr;

use itertools::Itertools;

use crate::daylib::Day;

#[derive(Debug)]
enum Shape {
    Rock,
    Paper,
    Scissors,
}

fn parse_shape(s: &str) -> Shape {
    match s {
        "A" | "X" => Shape::Rock,
        "B" | "Y" => Shape::Paper,
        "C" | "Z" => Shape::Scissors,
        _ => panic!("unknown shape {s}"),
    }
}

fn parse_line(s: &str) -> (Shape, Shape) {
    if let Some((theirs, mine)) = s.split_whitespace().collect_tuple() {
        (parse_shape(theirs), parse_shape(mine))
    } else {
        panic!("can't parse line {s}")
    }
}

#[allow(clippy::match_same_arms)]
fn score((theirs, mine): (Shape, Shape)) -> u32 {
    let shape_score: u32 = match mine {
        Shape::Rock => 1,
        Shape::Paper => 2,
        Shape::Scissors => 3,
    };
    let win_score: u32 = match (theirs, mine) {
        (Shape::Rock, Shape::Rock) => 3,
        (Shape::Rock, Shape::Paper) => 6,
        (Shape::Rock, Shape::Scissors) => 0,
        (Shape::Paper, Shape::Rock) => 0,
        (Shape::Paper, Shape::Paper) => 3,
        (Shape::Paper, Shape::Scissors) => 6,
        (Shape::Scissors, Shape::Rock) => 6,
        (Shape::Scissors, Shape::Paper) => 0,
        (Shape::Scissors, Shape::Scissors) => 3,
    };
    shape_score + win_score
}

#[allow(clippy::unnecessary_wraps)]
fn solve1(input: &str) -> color_eyre::Result<String> {
    Ok(input
        .split('\n')
        .map(parse_line)
        .map(score)
        .sum::<u32>()
        .to_string())
}

enum GameResult {
    Lose,
    Draw,
    Win,
}

#[derive(Debug, Clone)]
struct GameResultParseError(String);

impl fmt::Display for GameResultParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unknown game result {}", self.0)
    }
}

impl FromStr for GameResult {
    type Err = GameResultParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use GameResult::{Draw, Lose, Win};

        match s {
            "X" => Ok(Lose),
            "Y" => Ok(Draw),
            "Z" => Ok(Win),
            other => Err(GameResultParseError(other.to_string())),
        }
    }
}

fn parse_line_game_result(s: &str) -> (Shape, GameResult) {
    if let Some((theirs, result)) = s.split_whitespace().collect_tuple() {
        (parse_shape(theirs), GameResult::from_str(result).unwrap())
    } else {
        panic!("can't parse line {s}")
    }
}

#[allow(clippy::match_same_arms)]
fn infer_mine((theirs, result): (Shape, GameResult)) -> (Shape, Shape) {
    use GameResult::{Draw, Lose, Win};
    use Shape::{Paper, Rock, Scissors};

    let mine = match (&theirs, result) {
        (Rock, Lose) => Scissors,
        (Rock, Draw) => Rock,
        (Rock, Win) => Paper,
        (Paper, Lose) => Rock,
        (Paper, Draw) => Paper,
        (Paper, Win) => Scissors,
        (Scissors, Lose) => Paper,
        (Scissors, Draw) => Scissors,
        (Scissors, Win) => Rock,
    };

    (theirs, mine)
}

#[allow(clippy::unnecessary_wraps)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    Ok(input
        .split('\n')
        .map(parse_line_game_result)
        .map(infer_mine)
        .map(score)
        .sum::<u32>()
        .to_string())
}

pub(crate) const DAY: Day = Day {
    number: 2,
    part1: solve1,
    part2: solve2,
};
