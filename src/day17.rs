use std::collections::HashMap;
use std::iter;
use std::ops::{Index, IndexMut};

use color_eyre::eyre::eyre;
use itertools::Itertools;

use crate::daylib::Day;

const WIDTH: usize = 7;

// this is cheating a bit, we didn't show that it's impossible for the blocks
// to fall down the tower and affect lower levels, but hey ho if it works on test data
const MAX_DEPTH: usize = 100;

#[derive(Debug)]
enum Direction {
    Left,
    Right,
}

impl Direction {
    fn to_dx(&self) -> isize {
        match self {
            Direction::Left => -1,
            Direction::Right => 1,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Content {
    Empty,
    Stopped,
    Falling,
}

impl TryFrom<char> for Direction {
    type Error = color_eyre::Report;

    fn try_from(c: char) -> color_eyre::Result<Direction> {
        use Direction::{Left, Right};

        match c {
            '<' => Ok(Left),
            '>' => Ok(Right),
            other => Err(eyre!("unexpected direction {}", other)),
        }
    }
}

#[derive(Debug, Clone)]
struct Space {
    occupied: Vec<Content>,
    contents_height: usize,
    container_height: usize,
    rows_removed: usize,
}

impl Space {
    fn new() -> Self {
        Self {
            occupied: vec![],
            contents_height: 0,
            container_height: 0,
            rows_removed: 0,
        }
    }

    fn truncate(&mut self) {
        if self.contents_height < MAX_DEPTH {
            return;
        }
        let rows_to_remove = self.contents_height - MAX_DEPTH;

        drop(self.occupied.drain(..(rows_to_remove * WIDTH)));
        self.contents_height -= rows_to_remove;
        self.container_height -= rows_to_remove;
        self.rows_removed += rows_to_remove;
    }

    fn adjust_size(&mut self, new_shape: &Shape) {
        let new_container_height = self.contents_height + 3 + new_shape.height;
        if new_container_height < self.container_height {
            self.occupied.truncate(new_container_height * WIDTH);
        } else {
            self.occupied.extend(
                iter::repeat(Content::Empty)
                    .take((new_container_height - self.container_height) * WIDTH),
            );
        };
        self.container_height = new_container_height;

        self.truncate();
    }

    fn imprint(&mut self, shape: &Shape, (offset_x, offset_y): (usize, usize)) {
        self.contents_height = self.contents_height.max((self.container_height) - offset_y);

        shape
            .points
            .iter()
            .for_each(|(dx, dy)| self[(offset_x + dx, offset_y + dy)] = Content::Stopped);
    }

    #[allow(dead_code)]
    fn print(&self, falling: Option<(&Shape, (usize, usize))>) {
        let mut tmp_space = self.clone();

        if let Some((shape, (offset_x, offset_y))) = falling {
            for (dx, dy) in &shape.points {
                tmp_space[(offset_x + dx, offset_y + dy)] = Content::Falling;
            }
        }

        for y in (0..(tmp_space.occupied.len() / WIDTH)).rev() {
            println!(
                "{}\t|{}|",
                tmp_space.container_height - y - 1,
                tmp_space.occupied[(y * WIDTH)..((y + 1) * WIDTH)]
                    .iter()
                    .map(|x| {
                        match x {
                            Content::Empty => '.',
                            Content::Stopped => '#',
                            Content::Falling => '@',
                        }
                    })
                    .collect::<String>()
            );
        }

        println!();
    }
}

impl Index<(usize, usize)> for Space {
    type Output = Content;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        self.occupied
            .get(self.container_height.checked_sub(y + 1).unwrap() * WIDTH + x)
            .unwrap_or_else(|| panic!("out of bounds access at x {x} y {y}"))
    }
}

impl IndexMut<(usize, usize)> for Space {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        self.occupied
            .get_mut(self.container_height.checked_sub(y + 1).unwrap() * WIDTH + x)
            .unwrap_or_else(|| panic!("out of bounds access at x {x} y {y}"))
    }
}

#[derive(Debug, Clone)]
struct Shape {
    points: Vec<(usize, usize)>,
    height: usize,
}

impl Shape {
    fn new(points: &[(usize, usize)]) -> Self {
        let height = *points.iter().map(|(_x, y)| y).max().unwrap() + 1;
        Self {
            points: Vec::from(points),
            height,
        }
    }

    // y can't get negative
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn collides(&self, space: &Space, (tentative_x, tentative_y): (isize, usize)) -> bool {
        if tentative_x < 0 {
            // no need to check the points as offsets are vs top left corner
            return true;
        }
        self.points.iter().any(|(dx, dy)| {
            tentative_x as usize + dx > (WIDTH - 1)
                || tentative_y + dy == space.occupied.len() / WIDTH
                || space[(tentative_x as usize + dx, tentative_y + dy)] != Content::Empty
        })
    }
}

fn get_shapes() -> Vec<Shape> {
    vec![
        Shape::new(&[(0, 0), (1, 0), (2, 0), (3, 0)]),
        Shape::new(&[(1, 0), (0, 1), (1, 1), (2, 1), (1, 2)]),
        Shape::new(&[(2, 0), (2, 1), (0, 2), (1, 2), (2, 2)]),
        Shape::new(&[(0, 0), (0, 1), (0, 2), (0, 3)]),
        Shape::new(&[(0, 0), (0, 1), (1, 0), (1, 1)]),
    ]
}

#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
fn simulate(
    directions: impl Iterator<Item = Direction> + Clone,
    shapes: impl Iterator<Item = Shape> + Clone,
    n_rocks: usize,
) -> Space {
    let mut space = Space::new();

    let mut directions = directions.cycle();
    let mut shapes = shapes.cycle();

    for _rock in 0..n_rocks {
        let shape = shapes.next().unwrap();

        space.adjust_size(&shape);

        let mut offset_x = 2usize;
        let mut offset_y = 0usize;
        loop {
            let jet_dx = directions.next().unwrap().to_dx();

            let tentative_x = offset_x as isize + jet_dx;
            if !shape.collides(&space, (tentative_x, offset_y)) {
                assert!(tentative_x >= 0);
                offset_x = tentative_x as usize;
            }

            if shape.collides(&space, (offset_x as isize, offset_y + 1)) {
                break;
            }

            offset_y += 1;
        }

        space.imprint(&shape, (offset_x, offset_y));
    }

    space
}

#[allow(clippy::unnecessary_wraps)]
fn solve1(input: &str) -> color_eyre::Result<String> {
    let directions = input.chars().map(Direction::try_from).map(Result::unwrap);
    let shapes = get_shapes().into_iter();

    let space = simulate(directions, shapes, 2022);

    Ok((space.contents_height + space.rows_removed).to_string())
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct State {
    rows: Vec<u8>,
    jet_idx: u8,
    shape_idx: u8,
}

impl State {
    #[allow(clippy::cast_possible_truncation)]
    fn new(space: &Space, jet_idx: usize, shape_idx: usize) -> Self {
        State {
            rows: space
                .occupied
                .iter()
                .chunks(WIDTH)
                .into_iter()
                .map(|x| {
                    x.fold(0u8, |acc, c| match c {
                        Content::Empty => acc * 2,
                        Content::Stopped => acc * 2 + 1,
                        Content::Falling => unreachable!(),
                    })
                })
                .collect(),
            jet_idx: jet_idx as u8,
            shape_idx: shape_idx as u8,
        }
    }
}

#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
fn find_cycle(
    directions: impl Iterator<Item = Direction> + Clone,
    shapes: impl Iterator<Item = Shape> + Clone,
) -> (usize, usize) {
    let mut space = Space::new();

    let mut directions = directions.enumerate().cycle();
    let mut shapes = shapes.enumerate().cycle();

    let mut seen_states = HashMap::<State, usize>::new();

    for rock in 0.. {
        let (shape_idx, shape) = shapes.next().unwrap();

        space.adjust_size(&shape);

        let mut offset_x = 2usize;
        let mut offset_y = 0usize;
        let mut last_jet_idx;
        loop {
            let (jet_idx, jet) = directions.next().unwrap();
            last_jet_idx = jet_idx;

            let tentative_x = offset_x as isize + jet.to_dx();
            if !shape.collides(&space, (tentative_x, offset_y)) {
                assert!(tentative_x >= 0);
                offset_x = tentative_x as usize;
            }

            if shape.collides(&space, (offset_x as isize, offset_y + 1)) {
                break;
            }

            offset_y += 1;
        }

        space.imprint(&shape, (offset_x, offset_y));

        let new_state = State::new(&space, last_jet_idx, shape_idx);

        if let Some(prev_rock) = seen_states.insert(new_state, rock) {
            println!("found a repeat state after {rock} rocks, previous rock {prev_rock}, cycle length {}",
                rock - prev_rock);
            return (prev_rock, rock);
        }
    }

    unreachable!()
}

#[allow(clippy::unnecessary_wraps)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    let target_stones = 1_000_000_000_000;

    let directions = input.chars().map(Direction::try_from).map(Result::unwrap);
    let shapes = get_shapes().into_iter();

    let (cycle_start, cycle_end) = find_cycle(directions.clone(), shapes.clone());

    // -------

    let space0 = simulate(directions.clone(), shapes.clone(), cycle_start);
    let space1 = simulate(directions.clone(), shapes.clone(), cycle_end);

    let height_per_loop = (space1.rows_removed + space1.contents_height)
        - (space0.rows_removed + space0.contents_height);
    let num_loops = (target_stones - cycle_start) / (cycle_end - cycle_start);
    let after_loop = (target_stones - cycle_start) % (cycle_end - cycle_start);

    let unlooped_space = simulate(directions, shapes, cycle_end + after_loop);
    let unlooped_height = unlooped_space.contents_height + unlooped_space.rows_removed;

    Ok((unlooped_height + (num_loops - 1) * height_per_loop).to_string())
}

pub(crate) const DAY: Day = Day {
    number: 17,
    part1: solve1,
    part2: solve2,
};
