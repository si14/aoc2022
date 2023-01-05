use std::collections::{HashSet, VecDeque};
use std::iter;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

use color_eyre::eyre::eyre;
use itertools::Itertools;

use crate::daylib::Day;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Direction {
    U,
    D,
    L,
    R,
}

impl Direction {
    fn delta(self) -> (isize, isize) {
        use Direction::{D, L, R, U};

        match self {
            U => (0, -1),
            D => (0, 1),
            L => (-1, 0),
            R => (1, 0),
        }
    }
}

impl From<Direction> for char {
    fn from(value: Direction) -> Self {
        use Direction::{D, L, R, U};

        match value {
            U => '^',
            D => 'v',
            L => '<',
            R => '>',
        }
    }
}

impl TryFrom<char> for Direction {
    type Error = color_eyre::Report;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Direction::{D, L, R, U};

        match value {
            '^' => Ok(U),
            'v' => Ok(D),
            '<' => Ok(L),
            '>' => Ok(R),
            other => Err(eyre!("unexpected character {}", other)),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Loc {
    Ground,
    Blizzard(Direction),
    Wall,
}

impl From<Loc> for char {
    fn from(value: Loc) -> Self {
        use Loc::{Blizzard, Ground, Wall};

        match value {
            Ground => '.',
            Blizzard(d) => Self::from(d),
            Wall => '#',
        }
    }
}

impl TryFrom<char> for Loc {
    type Error = color_eyre::Report;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Loc::{Blizzard, Ground, Wall};

        match value {
            '.' => Ok(Ground),
            '#' => Ok(Wall),
            other => Ok(Blizzard(Direction::try_from(other)?)),
        }
    }
}

#[derive(Debug, Clone)]
struct Grid {
    d: Vec<Loc>,
    w: usize,
    h: usize,
    blizzards: Vec<(usize, usize, Direction)>,
    start: (usize, usize),
    end: (usize, usize),
}

impl Grid {
    #[allow(dead_code)]
    fn to_char(&self) -> CharGrid {
        CharGrid {
            d: self.d.iter().copied().map(char::from).collect(),
            w: self.w,
        }
    }
}

impl FromStr for Grid {
    type Err = color_eyre::Report;

    #[allow(clippy::match_on_vec_items)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let w = s
            .lines()
            .next()
            .ok_or(eyre!("at least one line should be present"))?
            .len();
        let h = s.lines().count();
        let d = s
            .lines()
            .flat_map(|l| l.chars().map(Loc::try_from))
            .collect::<Result<Vec<_>, _>>()?;

        let blizzards = (1..h)
            .flat_map(|y| (1..w).zip(iter::repeat(y)))
            .filter_map(|(x, y)| match d[y * w + x] {
                Loc::Blizzard(d) => Some((x, y, d)),
                _ => None,
            })
            .collect();

        let start_x = (0..w)
            .find(|i| d[*i] == Loc::Ground)
            .expect("must have entry in the first row");
        let end_x = (0..w)
            .find(|i| d[(w * (h - 1)) + *i] == Loc::Ground)
            .expect("must have exit in the last row");

        Ok(Grid {
            d,
            w,
            h,
            blizzards,
            start: (start_x, 0),
            end: (end_x, h - 1),
        })
    }
}

impl Index<(usize, usize)> for Grid {
    type Output = Loc;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        self.d
            .get(y * self.w + x)
            .unwrap_or_else(|| panic!("out of bounds access at x {x} y {y}"))
    }
}

impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        self.d
            .get_mut(y * self.w + x)
            .unwrap_or_else(|| panic!("out of bounds access at x {x} y {y}"))
    }
}

#[derive(Debug, Clone)]
struct CharGrid {
    d: Vec<char>,
    w: usize,
}

impl Index<(usize, usize)> for CharGrid {
    type Output = char;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        self.d
            .get(y * self.w + x)
            .unwrap_or_else(|| panic!("out of bounds access at x {x} y {y}"))
    }
}

impl IndexMut<(usize, usize)> for CharGrid {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        self.d
            .get_mut(y * self.w + x)
            .unwrap_or_else(|| panic!("out of bounds access at x {x} y {y}"))
    }
}

impl CharGrid {
    #[allow(dead_code)]
    fn print(&self) {
        println!(
            "{}",
            itertools::intersperse(
                self.d
                    .iter()
                    .chunks(self.w)
                    .into_iter()
                    .map(Iterator::collect),
                "\n".to_string()
            )
            .collect::<String>()
        );
    }
}

fn lcm(a: usize, b: usize) -> usize {
    for i in a.min(b).. {
        if i % a == 0 && i % b == 0 {
            return i;
        }
    }
    unreachable!()
}

#[test]
fn test_lcm() {
    assert_eq!(lcm(5, 3), 15);
    assert_eq!(lcm(4, 20), 20);
    assert_eq!(lcm(4, 6), 12);
}

struct GridCache {
    grids: Vec<Grid>,
}

impl GridCache {
    fn new(initial: Grid) -> Self {
        Self {
            grids: vec![initial],
        }
    }

    fn next_step(&mut self) -> &Grid {
        let prev_grid = self.grids.last().unwrap();

        let blizzards = prev_grid
            .blizzards
            .iter()
            .map(|(x, y, d)| {
                let (dx, dy) = d.delta();
                let (mut tx, mut ty) = (
                    x.checked_add_signed(dx).unwrap(),
                    y.checked_add_signed(dy).unwrap(),
                );
                if tx == 0 {
                    tx = prev_grid.w - 2;
                } else if tx == prev_grid.w - 1 {
                    tx = 1;
                } else if ty == 0 {
                    ty = prev_grid.h - 2;
                } else if ty == prev_grid.h - 1 {
                    ty = 1;
                }
                (tx, ty, *d)
            })
            .collect_vec();

        let mut grid = prev_grid.clone();

        prev_grid
            .blizzards
            .iter()
            .for_each(|(x, y, _)| grid[(*x, *y)] = Loc::Ground);
        blizzards
            .iter()
            .for_each(|(x, y, d)| grid[(*x, *y)] = Loc::Blizzard(*d));

        grid.blizzards = blizzards;

        self.grids.push(grid);

        self.grids.last().unwrap()
    }

    fn on(&mut self, minute: usize) -> &Grid {
        let grid = self.grids.first().unwrap();
        let minute = minute % lcm(grid.w, grid.h);
        if minute < self.grids.len() {
            &self.grids[minute]
        } else {
            for _ in 0..=(minute - self.grids.len()) {
                self.next_step();
            }
            self.grids.last().unwrap()
        }
    }
}

fn search(
    grid_cache: &mut GridCache,
    start: (usize, usize),
    end: (usize, usize),
    start_time: usize,
) -> (usize, Vec<Option<Direction>>) {
    #[derive(Debug)]
    struct Node {
        pos: (usize, usize),
        t: usize,
        path: Vec<Option<Direction>>,
    }

    let mut frontier = VecDeque::from([Node {
        pos: start,
        t: start_time,
        path: vec![],
    }]);
    let mut visited = HashSet::new();

    let mut result = None;
    while let Some(Node { pos, t, path }) = frontier.pop_front() {
        if visited.contains(&(pos, t)) {
            continue;
        }

        visited.insert((pos, t));

        if pos == end {
            result = Some((t, path));
            break;
        }

        let next_grid = grid_cache.on(t + 1);

        [
            Some(Direction::D),
            Some(Direction::R),
            Some(Direction::L),
            Some(Direction::U),
            None,
        ]
        .iter()
        .filter_map(|m| {
            let next_pos = match m {
                None => pos,
                Some(d) => {
                    let (dx, dy) = d.delta();
                    (pos.0.checked_add_signed(dx)?, pos.1.checked_add_signed(dy)?)
                }
            };
            if next_pos.1 > start.1.max(end.1) {
                // this removes the out-of-bounds access when we start at the end
                // and consider going down
                None
            } else if next_grid[next_pos] == Loc::Ground {
                Some((m, next_pos))
            } else {
                None
            }
        })
        .map(|(m, next_pos)| Node {
            pos: next_pos,
            t: t + 1,
            path: path.iter().copied().chain([*m]).collect(),
        })
        .for_each(|node| frontier.push_back(node));
    }

    result.expect("the loop above should find a path")
}

fn solve1(input: &str) -> color_eyre::Result<String> {
    let grid = input.parse::<Grid>()?;
    let mut cache = GridCache::new(grid.clone());

    let (duration, _steps) = search(&mut cache, grid.start, grid.end, 0);

    Ok(duration.to_string())
}

fn solve2(input: &str) -> color_eyre::Result<String> {
    let grid = input.parse::<Grid>()?;
    let mut cache = GridCache::new(grid.clone());

    let (duration1, _steps) = search(&mut cache, grid.start, grid.end, 0);
    let (duration2, _steps) = search(&mut cache, grid.end, grid.start, duration1);
    let (duration3, _steps) = search(&mut cache, grid.start, grid.end, duration2);

    Ok(duration3.to_string())
}

pub(crate) const DAY: Day = Day {
    number: 24,
    part1: solve1,
    part2: solve2,
};
