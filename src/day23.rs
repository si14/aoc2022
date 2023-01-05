use std::collections::{HashMap, VecDeque};
use std::iter;

use itertools::Itertools;

use crate::daylib::Day;

const GRID_WIDTH: isize = 1000;
const GRID_HEIGHT: isize = 1000;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Loc {
    Elf,
    Ground,
}

#[derive(Debug)]
struct Grid {
    d: Vec<Loc>,
    elves: Vec<(isize, isize)>,
    min_seen_x: isize,
    min_seen_y: isize,
    max_seen_x: isize,
    max_seen_y: isize,
}

impl Grid {
    fn parse(s: &str) -> Self {
        let mut g = Grid {
            d: vec![Loc::Ground; (GRID_HEIGHT * GRID_WIDTH) as usize],
            elves: vec![],
            min_seen_x: 0,
            min_seen_y: 0,
            max_seen_x: 0,
            max_seen_y: 0,
        };
        s.lines().enumerate().for_each(|(y, line)| {
            line.chars().enumerate().for_each(|(x, c)| {
                if c == '#' {
                    let x = isize::try_from(x).unwrap();
                    let y = isize::try_from(y).unwrap();
                    g.elves.push((x, y));
                    *g.get_mut((x, y)) = Loc::Elf;
                }
            });
        });

        g
    }

    fn print(&self) {
        let (bmin, bmax) = self.bounding_box();
        for y in bmin.1..=bmax.1 {
            for x in bmin.0..=bmax.0 {
                print!(
                    "{}",
                    match self.get((x, y)) {
                        Loc::Elf => '#',
                        Loc::Ground => '.',
                    }
                );
            }
            println!();
        }
    }

    fn bounding_box(&self) -> ((isize, isize), (isize, isize)) {
        let xs = self.elves.iter().map(|(x, _)| x);
        let ys = self.elves.iter().map(|(_, y)| y);
        (
            (*xs.clone().min().unwrap(), *ys.clone().min().unwrap()),
            (*xs.max().unwrap(), *ys.max().unwrap()),
        )
    }

    // can't use Index/IndexMut with negative coords
    fn get(&self, (x, y): (isize, isize)) -> Loc {
        let i = (y + GRID_HEIGHT / 2) * GRID_WIDTH + (x + GRID_WIDTH / 2);
        let i = usize::try_from(i).unwrap();
        *(self
            .d
            .get(i)
            .unwrap_or_else(|| panic!("out of bounds access at x {x} y {y}")))
    }

    fn get_mut(&mut self, (x, y): (isize, isize)) -> &mut Loc {
        self.min_seen_x = self.min_seen_x.min(x);
        self.min_seen_y = self.min_seen_y.min(y);
        self.max_seen_x = self.max_seen_x.max(x);
        self.max_seen_y = self.max_seen_y.max(y);

        let i = (y + GRID_HEIGHT / 2) * GRID_WIDTH + (x + GRID_WIDTH / 2);
        let i = usize::try_from(i).unwrap();

        self.d
            .get_mut(i)
            .unwrap_or_else(|| panic!("out of bounds access at x {x} y {y}"))
    }

    fn move_elf(&mut self, i: usize, d: Direction) {
        let from @ (x, y) = self.elves[i];
        let (dx, dy) = d.delta();
        let to = (x + dx, y + dy);

        assert_eq!(self.get(to), Loc::Ground);
        self.elves[i] = to;
        // can't use mem::swap b/c it won't update our bounding box
        *self.get_mut(from) = Loc::Ground;
        *self.get_mut(to) = Loc::Elf;
    }

    fn check(&self, i: usize, d: Direction) -> Option<(isize, isize)> {
        let (x, y) = self.elves[i];
        if d.check_deltas()
            .iter()
            .all(|(dx, dy)| self.get((x + dx, y + dy)) == Loc::Ground)
        {
            let (dx, dy) = d.delta();
            Some((x + dx, y + dy))
        } else {
            None
        }
    }

    fn free(&self, i: usize) -> bool {
        let (x, y) = self.elves[i];
        (-1..=1)
            .flat_map(|x| iter::repeat(x).zip(-1..=1))
            .all(|(dx, dy)| (dx == 0 && dy == 0) || self.get((x + dx, y + dy)) == Loc::Ground)
    }
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    N,
    S,
    W,
    E,
}

impl Direction {
    fn check_deltas(self) -> [(isize, isize); 3] {
        use Direction::{E, N, S, W};

        match self {
            N => [(-1, -1), (0, -1), (1, -1)],
            S => [(-1, 1), (0, 1), (1, 1)],
            W => [(-1, -1), (-1, 0), (-1, 1)],
            E => [(1, -1), (1, 0), (1, 1)],
        }
    }

    fn delta(self) -> (isize, isize) {
        use Direction::{E, N, S, W};

        match self {
            N => (0, -1),
            S => (0, 1),
            W => (-1, 0),
            E => (1, 0),
        }
    }
}

#[allow(clippy::unnecessary_wraps)]
fn solve1(input: &str) -> color_eyre::Result<String> {
    use Direction::{E, N, S, W};

    let mut grid = Grid::parse(input);
    grid.print();

    let mut directions = VecDeque::from([N, S, W, E]);
    for _round in 0..10 {
        let proposals: HashMap<(isize, isize), Vec<(usize, Direction)>> = grid
            .elves
            .iter()
            .enumerate()
            .filter(|(i, _)| !grid.free(*i))
            .filter_map(|(i, _)| {
                directions
                    .iter()
                    .find_map(|d| grid.check(i, *d).map(|coords| (d, coords)))
                    .map(|(d, to)| (to, (i, *d)))
            })
            .into_group_map();
        proposals
            .into_iter()
            .filter(|(_to, v)| v.len() == 1)
            .for_each(|(_to, v)| {
                let (i, d) = v[0];
                grid.move_elf(i, d);
            });
        directions.rotate_left(1);
    }

    let (bmin, bmax) = grid.bounding_box();
    let n_ground = (bmax.0.abs_diff(bmin.0) + 1) * (bmax.1.abs_diff(bmin.1) + 1) - grid.elves.len();
    Ok(n_ground.to_string())
}

#[allow(clippy::unnecessary_wraps)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    use Direction::{E, N, S, W};

    let mut grid = Grid::parse(input);
    grid.print();

    let mut directions = VecDeque::from([N, S, W, E]);
    let mut final_round = None;
    for round in 0.. {
        let proposals: HashMap<(isize, isize), Vec<(usize, Direction)>> = grid
            .elves
            .iter()
            .enumerate()
            .filter(|(i, _)| !grid.free(*i))
            .filter_map(|(i, _)| {
                directions
                    .iter()
                    .find_map(|d| grid.check(i, *d).map(|coords| (d, coords)))
                    .map(|(d, to)| (to, (i, *d)))
            })
            .into_group_map();

        if proposals.is_empty() {
            final_round = Some(round);
            break;
        }

        proposals
            .into_iter()
            .filter(|(_to, v)| v.len() == 1)
            .for_each(|(_to, v)| {
                let (i, d) = v[0];
                grid.move_elf(i, d);
            });
        directions.rotate_left(1);
    }

    Ok((final_round.unwrap() + 1).to_string())
}

pub(crate) const DAY: Day = Day {
    number: 23,
    part1: solve1,
    part2: solve2,
};
