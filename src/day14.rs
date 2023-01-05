use std::ops::{Index, IndexMut};
use std::{cmp, iter, usize};

use itertools::Itertools;

use crate::daylib::Day;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Material {
    Air,
    Rock,
    Sand,
    Abyss,
    Floor,
}

const MAX_WIDTH: usize = 1000;

#[derive(Debug)]
struct Grid {
    data: Vec<Material>,
    w: usize,
    h: usize,
    min_seen_x: usize,
    max_seen_x: usize,
    has_floor: bool,
    sand_source: (usize, usize),
}

impl Grid {
    fn new(
        sand_source: (usize, usize),
        traces: &Vec<Vec<(usize, usize)>>,
        has_floor: bool,
    ) -> Self {
        let points = traces
            .iter()
            .flatten()
            .chain(iter::once(&sand_source))
            .collect::<Vec<_>>();
        let min_seen_x = *points.iter().map(|(x, _)| x).min().unwrap();
        let max_seen_x = *points.iter().map(|(x, _)| x).max().unwrap();
        let max_seen_y = *points.iter().map(|(_, y)| y).max().unwrap();

        let w = MAX_WIDTH;
        // +1 to account for size vs value, +2 to add floor/abyss
        let h = max_seen_y + 3;
        let mut grid = Grid {
            data: vec![Material::Air; w * h],
            w,
            h,
            min_seen_x,
            max_seen_x,
            has_floor,
            sand_source,
        };

        for x in 0..MAX_WIDTH {
            // set .data directly to skip max/min_seen checks
            grid.data[grid.w * (grid.h - 1) + x] = if has_floor {
                Material::Floor
            } else {
                Material::Abyss
            };
        }

        for trace in traces {
            for ((xa, ya), (xb, yb)) in trace.iter().tuple_windows() {
                if xa == xb {
                    for y in cmp::min(*ya, *yb)..=cmp::max(*ya, *yb) {
                        grid[(*xa, y)] = Material::Rock;
                    }
                } else {
                    assert_eq!(ya, yb);
                    for x in cmp::min(*xa, *xb)..=cmp::max(*xa, *xb) {
                        grid[(x, *ya)] = Material::Rock;
                    }
                }
            }
        }

        grid
    }

    fn format(&self) -> String {
        Itertools::intersperse(
            self.data.chunks(self.w).map(|line| {
                use Material::{Abyss, Air, Floor, Rock, Sand};

                line.iter()
                    .skip(self.min_seen_x - 1)
                    .take(self.max_seen_x - self.min_seen_x + 2)
                    .map(|m| match m {
                        Rock => '#',
                        Air => '.',
                        Sand => 'o',
                        Abyss => 'A',
                        Floor => 'F',
                    })
                    .collect::<String>()
            }),
            "\n".to_string(),
        )
        .collect::<String>()
    }
}

impl Index<(usize, usize)> for Grid {
    type Output = Material;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        self.data
            .get(y * self.w + x)
            .unwrap_or_else(|| panic!("out of bounds access at x {x} y {y}"))
    }
}

impl IndexMut<(usize, usize)> for Grid {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        if x < self.min_seen_x {
            self.min_seen_x = x;
        } else if x > self.max_seen_x {
            self.max_seen_x = x;
        }

        self.data
            .get_mut(y * self.w + x)
            .unwrap_or_else(|| panic!("out of bounds access at x {x} y {y}"))
    }
}

#[derive(Debug, Copy, Clone)]
enum StepResult {
    Rest(usize, usize),
    Move(usize, usize),
    Abyss,
}

fn step(grid: &Grid, _sand @ (x, y): (usize, usize)) -> StepResult {
    use StepResult::{Abyss, Move, Rest};

    if !grid.has_floor && y == grid.h - 2 {
        return Abyss;
    }

    if let Some((new_x, new_y)) = vec![(0, 1), (-1, 1), (1, 1)]
        .iter()
        .map(|(dx, dy)| {
            (
                x.checked_add_signed(*dx).unwrap(),
                y.checked_add_signed(*dy).unwrap(),
            )
        })
        .find(|new_pos| grid[*new_pos] == Material::Air)
    {
        Move(new_x, new_y)
    } else {
        Rest(x, y)
    }
}

const SAND_SOURCE: (usize, usize) = (500, 0);

fn parse_traces(s: &str) -> color_eyre::Result<Vec<Vec<(usize, usize)>>> {
    s.lines()
        .map(|line| {
            line.split(" -> ")
                .map(|pair_s| {
                    let mut it = pair_s.split(',');
                    if let (Some(a_s), Some(b_s), None) = (it.next(), it.next(), it.next()) {
                        Ok::<_, color_eyre::Report>((a_s.parse::<usize>()?, b_s.parse::<usize>()?))
                    } else {
                        panic!("unexpected pair format {pair_s:?}")
                    }
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()
}

fn simulate(mut grid: Grid) -> Grid {
    loop {
        use StepResult::{Abyss, Move, Rest};

        let mut result = Move(SAND_SOURCE.0, SAND_SOURCE.1);
        while let Move(new_x, new_y) = result {
            result = step(&grid, (new_x, new_y));
        }

        match result {
            // part 1, stop when stuff starts falling into the abyss
            Abyss => break,
            // part 2, stop when there the source is blocked
            Rest(x, y) if (x, y) == grid.sand_source => {
                grid[(x, y)] = Material::Sand;
                break;
            }
            Rest(x, y) => grid[(x, y)] = Material::Sand,
            Move(_, _) => unreachable!(),
        }
    }

    grid
}

fn solve1(input: &str) -> color_eyre::Result<String> {
    let traces = parse_traces(input)?;

    let mut grid = Grid::new(SAND_SOURCE, &traces, false);
    println!("pre-simulation grid:\n{}\n", grid.format());

    grid = simulate(grid);
    println!("post-simulation grid:\n{}\n", grid.format());

    Ok(grid
        .data
        .iter()
        .filter(|m| **m == Material::Sand)
        .count()
        .to_string())
}

fn solve2(input: &str) -> color_eyre::Result<String> {
    let traces = parse_traces(input)?;

    let mut grid = Grid::new(SAND_SOURCE, &traces, true);
    println!("pre-simulation grid:\n{}\n", grid.format());

    grid = simulate(grid);
    println!("post-simulation grid:\n{}\n", grid.format());

    Ok(grid
        .data
        .iter()
        .filter(|m| **m == Material::Sand)
        .count()
        .to_string())
}

pub(crate) const DAY: Day = Day {
    number: 14,
    part1: solve1,
    part2: solve2,
};
