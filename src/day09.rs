use std::collections::HashSet;
use std::iter;
use std::str::FromStr;

use color_eyre::eyre::eyre;
use itertools::Itertools;

use crate::daylib::Day;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
struct Position {
    x: isize,
    y: isize,
}

impl Position {
    fn step(&self, d: Direction) -> Position {
        use Direction::{Down, Left, Right, Up};

        let Position { x, y } = self;
        match d {
            Up => Position { x: *x, y: y + 1 },
            Down => Position { x: *x, y: y - 1 },
            Left => Position { x: x - 1, y: *y },
            Right => Position { x: x + 1, y: *y },
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct State<const N: usize> {
    // rope.0 is head
    rope: [Position; N],
}

fn new_tail(hx: isize, hy: isize, tx: isize, ty: isize) -> (isize, isize) {
    if (tx - hx).abs() > 1 {
        assert_eq!((tx - hx).abs(), 2);
        assert!((ty - hy).abs() <= 2);
        // 0 -> 0, 1 -> 1, 2 -> 1
        (
            tx + ((hx - tx) / 2),
            if (ty - hy).abs() < 2 {
                hy
            } else {
                (ty + hy) / 2
            },
        )
    } else if (ty - hy).abs() > 1 {
        assert_eq!((ty - hy).abs(), 2);
        assert!((tx - hx).abs() <= 2);
        (
            if (tx - hx).abs() < 2 {
                hx
            } else {
                (tx + hx) / 2
            },
            ty + ((hy - ty) / 2),
        )
    } else {
        (tx, ty)
    }
}

impl<const N: usize> State<N> {
    fn reconcile(&mut self) {
        for i in 0..(N - 1) {
            let head = self.rope[i];
            let tail = self.rope[i + 1];
            let (new_x, new_y) = new_tail(head.x, head.y, tail.x, tail.y);
            self.rope[i + 1] = Position { x: new_x, y: new_y };
        }
    }

    fn apply(&self, d: Direction) -> State<N> {
        let mut new_rope = self.rope;
        new_rope[0] = new_rope[0].step(d);
        let mut new_state = State { rope: new_rope };
        new_state.reconcile();
        new_state
    }
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl FromStr for Direction {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Direction::{Down, Left, Right, Up};
        match s {
            "U" => Ok(Up),
            "D" => Ok(Down),
            "L" => Ok(Left),
            "R" => Ok(Right),
            other => Err(eyre!("unknown direction {other:?}")),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Command {
    d: Direction,
    n: usize,
}

impl FromStr for Command {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split(' ');
        if let (Some(d_s), Some(n_s), None) = (iter.next(), iter.next(), iter.next()) {
            Ok(Command {
                d: d_s.parse()?,
                n: n_s.parse()?,
            })
        } else {
            Err(eyre!("malformed command {s:?}"))
        }
    }
}

#[allow(dead_code, clippy::cast_sign_loss, clippy::cast_possible_truncation)]
fn draw_states<const N: usize>(states: &Vec<State<N>>) {
    let max_x = states
        .iter()
        .flat_map(|s| s.rope)
        .map(|p| p.x)
        .inspect(|x| assert!(*x >= 0))
        .max()
        .unwrap() as usize;
    let max_y = states
        .iter()
        .flat_map(|s| s.rope)
        .map(|p| p.y)
        .inspect(|y| assert!(*y >= 0))
        .max()
        .unwrap() as usize;

    for state in states {
        let mut screen =
            iter::repeat_with(|| iter::repeat('*').take(max_x + 1).collect::<Vec<_>>())
                .take(max_y + 1)
                .collect::<Vec<_>>();

        for piece in (0..N).rev() {
            let glyph = match piece {
                0 => 'H',
                1 if N == 2 => 'T',
                other => char::from_digit(other as u32, 10).unwrap(),
            };
            screen[state.rope[piece].y as usize][state.rope[piece].x as usize] = glyph;
        }

        println!(
            "{}\n\n",
            Itertools::intersperse(
                screen
                    .iter()
                    .map(|line| line.iter().collect::<String>())
                    .rev(),
                "\n".to_string()
            )
            .collect::<String>()
        );
    }
}

fn model<const N: usize>(commands: &[Command]) -> Vec<State<N>> {
    let mut states = vec![State {
        rope: [Position { x: 0, y: 0 }; N],
    }];

    states.extend(
        commands
            .iter()
            //.inspect(|x| println!("command: {x:#?}"))
            .flat_map(|c| iter::repeat(c.d).take(c.n))
            //.inspect(|x| println!("direction: {x:#?}"))
            .scan(
                State {
                    rope: [Position { x: 0, y: 0 }; N],
                },
                |state, d| {
                    *state = state.apply(d);
                    Some(*state)
                },
            ), //.inspect(|x| println!("state: {x:#?}")),
    );

    states
}

#[allow(clippy::unnecessary_wraps)]
fn solve1(input: &str) -> color_eyre::Result<String> {
    let commands = input
        .lines()
        //.take(1)
        .map(Command::from_str)
        .map(Result::unwrap)
        .collect_vec();
    let states = model::<2>(&commands);

    //draw_states(&states);

    Ok(states
        .iter()
        .map(|s| *s.rope.last().unwrap())
        .collect::<HashSet<_>>()
        .len()
        .to_string())
}

#[allow(clippy::unnecessary_wraps)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    let commands = input
        .lines()
        //.take(1)
        .map(Command::from_str)
        .map(Result::unwrap)
        .collect_vec();
    let states = model::<10>(&commands);

    //draw_states(&states);

    Ok(states
        .iter()
        .map(|s| *s.rope.last().unwrap())
        .collect::<HashSet<_>>()
        .len()
        .to_string())
}

pub(crate) const DAY: Day = Day {
    number: 9,
    part1: solve1,
    part2: solve2,
};
