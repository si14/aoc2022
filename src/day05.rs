use std::str::FromStr;

use color_eyre::eyre::eyre;
use itertools::Itertools;

use crate::daylib::Day;

#[derive(Debug)]
struct Crate(char);

impl From<char> for Crate {
    fn from(c: char) -> Self {
        Self(c)
    }
}

impl From<Crate> for char {
    fn from(c: Crate) -> Self {
        c.0
    }
}

#[derive(Debug)]
struct Move {
    from: usize,
    to: usize,
    n: usize,
}

impl FromStr for Move {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let iter = s.split(' ');
        let Some(("move", n_s, "from", from_s, "to", to_s)) = iter.collect_tuple() else {
            return Err(eyre!("malformed Move: {s:?}"))
        };

        Ok(Move {
            from: from_s.parse::<usize>()? - 1,
            to: to_s.parse::<usize>()? - 1,
            n: n_s.parse::<usize>()?,
        })
    }
}

// bottom-up
#[derive(Debug)]
struct Stacks(Vec<Vec<Crate>>);

impl Stacks {
    fn bottom_crates(self) -> String {
        self.0
            .into_iter()
            .map(|stack| stack.last().unwrap().0)
            .collect()
    }

    fn apply_9000(&mut self, moves: Vec<Move>) {
        for m in moves {
            for _ in 0..m.n {
                let c = self.0[m.from].pop().expect("invalid move?");
                self.0[m.to].push(c);
            }
        }
    }

    fn apply_9001(&mut self, moves: Vec<Move>) {
        for m in moves {
            let mut buf = Vec::<Crate>::with_capacity(m.n);

            for _ in 0..m.n {
                buf.push(self.0[m.from].pop().expect("invalid move?"));
            }

            for _ in 0..m.n {
                self.0[m.to].push(buf.pop().expect("invalid move?"));
            }
        }
    }
}

impl FromStr for Stacks {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut stacks = Stacks(vec![]);
        for _ in 0..((s.lines().next().unwrap().len() + 1) / 4) {
            stacks.0.push(Vec::new());
        }

        'outer: for line in s.lines() {
            for (i, mut chunk) in line.chars().chunks(4).into_iter().enumerate() {
                match (chunk.next(), chunk.next(), chunk.next()) {
                    (Some('['), Some(c), Some(']')) => {
                        stacks.0[i].push(Crate::from(c));
                        continue;
                    }
                    (Some(' '), Some(' '), Some(' ')) => continue,
                    // this is the last line of the diagram
                    (Some(' '), Some('1'), Some(' ')) => break 'outer,
                    other => return Err(eyre!("malformed line {line:?}, encountered {other:?}")),
                }
            }
        }
        for stack in &mut stacks.0 {
            stack.reverse();
        }
        Ok(stacks)
    }
}

fn solve1(input: &str) -> color_eyre::Result<String> {
    let Some((stack_part, moves_part)) = input.split("\n\n").collect_tuple() else {
        return Err(eyre!("can't split the file into stack and moves parts"))
    };

    let moves = moves_part
        .lines()
        .map(Move::from_str)
        .collect::<Result<Vec<_>, _>>()?;

    let mut stacks = Stacks::from_str(stack_part)?;

    stacks.apply_9000(moves);

    Ok(stacks.bottom_crates())
}

fn solve2(input: &str) -> color_eyre::Result<String> {
    let Some((stack_part, moves_part)) = input.split("\n\n").collect_tuple() else {
        return Err(eyre!("can't split the file into stack and moves parts"))
    };

    let moves = moves_part
        .lines()
        .map(Move::from_str)
        .collect::<Result<Vec<_>, _>>()?;

    let mut stacks = Stacks::from_str(stack_part)?;

    stacks.apply_9001(moves);

    Ok(stacks.bottom_crates())
}

pub(crate) const DAY: Day = Day {
    number: 5,
    part1: solve1,
    part2: solve2,
};
