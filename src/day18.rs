use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

use crate::daylib::Day;

const GRID_SIZE: usize = 22;

struct Grid {
    d: [bool; GRID_SIZE * GRID_SIZE * GRID_SIZE],
}

impl Index<(usize, usize, usize)> for Grid {
    type Output = bool;

    fn index(&self, (x, y, z): (usize, usize, usize)) -> &Self::Output {
        self.d
            .get(x * GRID_SIZE * GRID_SIZE + y * GRID_SIZE + z)
            .unwrap_or_else(|| panic!("out of bounds access at x {x} y {y} z {z}"))
    }
}

impl IndexMut<(usize, usize, usize)> for Grid {
    fn index_mut(&mut self, (x, y, z): (usize, usize, usize)) -> &mut Self::Output {
        self.d
            .get_mut(x * GRID_SIZE * GRID_SIZE + y * GRID_SIZE + z)
            .unwrap_or_else(|| panic!("out of bounds access at x {x} y {y} z {z}"))
    }
}

const DELTAS: [(isize, isize, isize); 6] = [
    (0, 0, 1),
    (0, 0, -1),
    (0, 1, 0),
    (0, -1, 0),
    (1, 0, 0),
    (-1, 0, 0),
];

#[allow(clippy::unnecessary_wraps)]
fn solve1(input: &str) -> color_eyre::Result<String> {
    let mut grid = Grid {
        d: [false; GRID_SIZE * GRID_SIZE * GRID_SIZE],
    };

    let cubes = input
        .lines()
        .map(|l| {
            let mut it = l.split(',');
            if let (Some(x_s), Some(y_s), Some(z_s), None) =
                (it.next(), it.next(), it.next(), it.next())
            {
                (
                    x_s.parse().unwrap(),
                    y_s.parse().unwrap(),
                    z_s.parse().unwrap(),
                )
            } else {
                panic!("weird line {l}");
            }
        })
        .collect::<Vec<(usize, usize, usize)>>();

    for c in &cubes {
        grid[*c] = true;
    }

    Ok(cubes
        .iter()
        .map(|(x, y, z)| {
            6 - DELTAS
                .iter()
                .copied()
                .map(|(dx, dy, dz)| {
                    if let (Some(x2), Some(y2), Some(z2)) = (
                        x.checked_add_signed(dx),
                        y.checked_add_signed(dy),
                        z.checked_add_signed(dz),
                    ) {
                        usize::from(grid[(x2, y2, z2)])
                    } else {
                        // negative coordinates
                        0
                    }
                })
                .sum::<usize>()
        })
        .sum::<usize>()
        .to_string())
}

#[allow(clippy::unnecessary_wraps)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    let mut grid = Grid {
        d: [false; GRID_SIZE * GRID_SIZE * GRID_SIZE],
    };
    let mut grid_seen = Grid {
        d: [false; GRID_SIZE * GRID_SIZE * GRID_SIZE],
    };

    let cubes = input
        .lines()
        .map(|l| {
            let mut it = l.split(',');
            if let (Some(x_s), Some(y_s), Some(z_s), None) =
                (it.next(), it.next(), it.next(), it.next())
            {
                (
                    // shift them a bit to BFS on the outside
                    x_s.parse::<usize>().unwrap() + 1,
                    y_s.parse::<usize>().unwrap() + 1,
                    z_s.parse::<usize>().unwrap() + 1,
                )
            } else {
                panic!("weird line {l}");
            }
        })
        .collect::<Vec<(usize, usize, usize)>>();

    for c in &cubes {
        grid[*c] = true;
    }

    let mut q = VecDeque::<(usize, usize, usize)>::new();
    q.push_back((0, 0, 0));

    let mut surfaces = 0;

    while let Some((x, y, z)) = q.pop_front() {
        for neighbour in DELTAS
            .iter()
            .filter_map(|(dx, dy, dz)| {
                if let (Some(x2), Some(y2), Some(z2)) = (
                    x.checked_add_signed(*dx),
                    y.checked_add_signed(*dy),
                    z.checked_add_signed(*dz),
                ) {
                    Some((x2, y2, z2))
                } else {
                    None
                }
            })
            .filter(|(x, y, z)| *x < GRID_SIZE && *y < GRID_SIZE && *z < GRID_SIZE)
        {
            if grid[neighbour] {
                surfaces += 1;
            } else if !grid_seen[neighbour] {
                q.push_back(neighbour);
                grid_seen[neighbour] = true;
            }
        }
    }

    Ok(surfaces.to_string())
}

pub(crate) const DAY: Day = Day {
    number: 18,
    part1: solve1,
    part2: solve2,
};
