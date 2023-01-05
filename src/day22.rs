use std::collections::HashMap;
use std::iter;
use std::ops::{Index, IndexMut};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::digit1;
use nom::combinator::{map, value};
use nom::error::Error as NomError;
use nom::multi::many1;
use nom::Finish;

use crate::daylib::Day;

#[derive(Debug)]
struct Grid<T> {
    d: Vec<T>,
    w: usize,
    h: usize,
}

impl<T> Index<(usize, usize)> for Grid<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        self.d
            .get(y * self.w + x)
            .unwrap_or_else(|| panic!("out of bounds access at x {x} y {y}"))
    }
}

impl<T> IndexMut<(usize, usize)> for Grid<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        self.d
            .get_mut(y * self.w + x)
            .unwrap_or_else(|| panic!("out of bounds access at x {x} y {y}"))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Tile {
    Open,
    Wall,
    Oob,
}

impl Grid<Tile> {
    fn parse(s: &str) -> Self {
        use Tile::{Oob, Open, Wall};

        let w = s.lines().map(str::len).max().unwrap();
        let h = s.lines().count();
        let d = s
            .lines()
            .flat_map(|l| {
                l.chars()
                    .map(|c| match c {
                        ' ' => Oob,
                        '.' => Open,
                        '#' => Wall,
                        _ => panic!("unexpected char {c}"),
                    })
                    .chain(iter::repeat(Oob))
                    .take(w)
            })
            .collect::<Vec<_>>();
        assert_eq!(d.len(), w * h);

        Self { d, w, h }
    }

    #[allow(dead_code)]
    fn print(&self, path: Option<&Vec<Position>>) {
        use Tile::{Oob, Open, Wall};

        let mut char_grid = Grid::<char> {
            d: self
                .d
                .iter()
                .map(|t| match t {
                    Open => '.',
                    Wall => '#',
                    Oob => ' ',
                })
                .collect(),
            w: self.w,
            h: self.h,
        };

        if let Some(path) = path {
            use Direction::{D, L, R, U};

            for p in path {
                char_grid[(p.x, p.y)] = match p.d {
                    U => '^',
                    R => '>',
                    D => 'v',
                    L => '<',
                }
            }

            let last = path.last().unwrap();
            char_grid[(last.x, last.y)] = '*';
        }

        for chunk in char_grid.d.chunks(self.w) {
            println!("{}", chunk.iter().collect::<String>());
        }
    }
}

// moves to wherever we're supposed to be if we're in Position and make one step forward
type TeleportMap = HashMap<Position, Position>;

fn calc_teleports_part1(map: &Grid<Tile>) -> TeleportMap {
    use Direction::{D, L, R, U};

    let mut t_map = TeleportMap::new();

    for y in 0..map.h {
        let mut a = None;
        let mut b = None;
        for x in 0..map.w {
            if a.is_none() {
                if map[(x, y)] != Tile::Oob {
                    a = Some(x);
                }
            } else if map[(x, y)] == Tile::Oob {
                b = Some(x - 1);
                break;
            }
        }
        let a = a.unwrap();
        let b = b.unwrap_or(map.w - 1);
        t_map.insert(Position::new(a, y, L), Position::new(b, y, L));
        t_map.insert(Position::new(b, y, R), Position::new(a, y, R));
    }

    for x in 0..map.w {
        let mut a = None;
        let mut b = None;
        for y in 0..map.h {
            if a.is_none() {
                if map[(x, y)] != Tile::Oob {
                    a = Some(y);
                }
            } else if map[(x, y)] == Tile::Oob {
                b = Some(y - 1);
                break;
            }
        }
        let a = a.unwrap();
        let b = b.unwrap_or(map.h - 1);
        t_map.insert(Position::new(x, a, U), Position::new(x, b, U));
        t_map.insert(Position::new(x, b, D), Position::new(x, a, D));
    }

    t_map
}

fn calc_teleports_part2(map: &Grid<Tile>) -> TeleportMap {
    use Direction::{D, L, R, U};

    fn face_coords(
        map: &Grid<Tile>,
        faces: &[(usize, usize)],
        face: usize,
        side: Direction,
    ) -> Vec<(usize, usize)> {
        let face_size = map.w / 3;
        // top left corner
        let face_x = faces[face].0 * face_size;
        let face_y = faces[face].1 * face_size;
        let (shift_x, shift_y, dx, dy) = match side {
            U => (0, 0, 1, 0),
            D => (0, face_size - 1, 1, 0),
            L => (0, 0, 0, 1),
            R => (face_size - 1, 0, 0, 1),
        };
        (0..face_size)
            .map(move |i| (face_x + shift_x + dx * i, face_y + shift_y + dy * i))
            .collect()
    }

    // top left corner in multiples of face size
    let faces: [(usize, usize); 6] = [(1, 0), (2, 0), (1, 1), (0, 2), (1, 2), (0, 3)];

    // the last tuple member is "is the stitch inverted", for some faces we need to invert the
    // face coordinates
    // example of an inverse: ((1, U), (6, R), true) -> ((6, L), (1, D), true)
    #[allow(clippy::type_complexity)]
    let transitions: Vec<((usize, Direction), (usize, Direction), bool)> = [
        ((0, U), (5, R), false),
        ((0, L), (3, R), true),
        ((1, U), (5, U), false),
        ((1, R), (4, L), true),
        ((2, L), (3, D), false),
        ((2, R), (1, U), false),
        ((4, D), (5, L), false),
    ]
    .into_iter()
    .flat_map(|orig @ ((f1, d1), (f2, d2), inv)| {
        vec![orig, ((f2, d2.inverse()), (f1, d1.inverse()), inv)]
    })
    .collect::<Vec<_>>();

    transitions
        .iter()
        .flat_map(|((f1, d1), (f2, d2), inv)| {
            let face1_coords = face_coords(map, &faces, *f1, *d1);
            // we need to invert the direction because we arrive at the opposite side
            // (e.g. if we're facing left, we arrive at the right side)
            let mut face2_coords = face_coords(map, &faces, *f2, d2.inverse());
            if *inv {
                face2_coords.reverse();
            }
            face1_coords
                .into_iter()
                .zip(face2_coords.into_iter())
                //.inspect(|x| println!("{x:?} f1 {} d1 {:?} f2 {} d2 {:?}", *f1, *d1, *f2, *d2,))
                .map(|((x1, y1), (x2, y2))| {
                    (Position::new(x1, y1, *d1), Position::new(x2, y2, *d2))
                })
        })
        .collect::<TeleportMap>()
}

#[derive(Debug, Copy, Clone)]
enum PathPart {
    Move(usize),
    L,
    R,
}

fn parse_path(s: &str) -> Vec<PathPart> {
    use PathPart::{Move, L, R};

    many1(alt((
        value(L, tag::<_, _, NomError<_>>("L")),
        value(R, tag("R")),
        map(digit1, |s: &str| Move(s.parse().unwrap())),
    )))(s)
    .finish()
    .unwrap()
    .1
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
enum Direction {
    U,
    R,
    D,
    L,
}

impl Direction {
    fn delta(self) -> (isize, isize) {
        use Direction::{D, L, R, U};

        match self {
            U => (0, -1),
            R => (1, 0),
            D => (0, 1),
            L => (-1, 0),
        }
    }

    fn inverse(self) -> Self {
        use Direction::{D, L, R, U};

        match self {
            U => D,
            R => L,
            D => U,
            L => R,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
struct Position {
    x: usize,
    y: usize,
    d: Direction,
}

impl Position {
    fn new(x: usize, y: usize, d: Direction) -> Self {
        Self { x, y, d }
    }

    fn fresh(map: &Grid<Tile>) -> Self {
        let x = map
            .d
            .iter()
            .enumerate()
            .take(map.w)
            .find(|(_, x)| Tile::Open.eq(x))
            .expect("should have one open tile in the first row")
            .0;
        Self {
            x,
            y: 0,
            d: Direction::R,
        }
    }

    fn apply(&mut self, p: PathPart, map: &Grid<Tile>, teleports: &TeleportMap) {
        use Direction::{D, L, R, U};

        if let PathPart::Move(n) = p {
            for _ in 0..n {
                let tentative = if let Some(new_position) = teleports.get(self) {
                    *new_position
                } else {
                    Position {
                        x: self.x.checked_add_signed(self.d.delta().0).unwrap(),
                        y: self.y.checked_add_signed(self.d.delta().1).unwrap(),
                        ..*self
                    }
                };

                match map[(tentative.x, tentative.y)] {
                    Tile::Open => {
                        self.x = tentative.x;
                        self.y = tentative.y;
                        self.d = tentative.d;
                    }
                    Tile::Wall => (),
                    Tile::Oob => unreachable!(),
                }
            }
        } else {
            #[allow(clippy::match_same_arms)]
            match (self.d, p) {
                (U, PathPart::L) => self.d = L,
                (U, PathPart::R) => self.d = R,
                (R, PathPart::L) => self.d = U,
                (R, PathPart::R) => self.d = D,
                (D, PathPart::L) => self.d = R,
                (D, PathPart::R) => self.d = L,
                (L, PathPart::L) => self.d = D,
                (L, PathPart::R) => self.d = U,
                _ => unreachable!(),
            }
        }
    }
}

#[allow(clippy::unnecessary_wraps)]
fn solve1(input: &str) -> color_eyre::Result<String> {
    let mut it = input.split("\n\n");
    let map = Grid::parse(it.next().expect("map should be there"));
    let path = parse_path(it.next().expect("path should be there"));
    let teleport_map = calc_teleports_part1(&map);

    let positions = path
        .iter()
        .scan(Position::fresh(&map), |position, path_part| {
            position.apply(*path_part, &map, &teleport_map);
            Some(*position)
        })
        .collect::<Vec<_>>();

    //map.print(Some(&positions));

    let last = positions.last().unwrap();
    {
        use Direction::{D, L, R, U};

        Ok(((last.y + 1) * 1000
            + (last.x + 1) * 4
            + match last.d {
                R => 0,
                D => 1,
                L => 2,
                U => 3,
            })
        .to_string())
    }
}

#[allow(clippy::unnecessary_wraps)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    let mut it = input.split("\n\n");
    let map = Grid::parse(it.next().expect("map should be there"));
    let path = parse_path(it.next().expect("path should be there"));
    let teleport_map = calc_teleports_part2(&map);

    // println!(
    //     "{:#?}",
    //     teleport_map
    //         .iter()
    //         .filter(|(k, v)| k.y == 0 && k.d == Direction::U)
    //         .sorted_by_key(|(k, _)| k.x)
    //         .collect::<Vec<_>>()
    // );

    let positions = path
        .iter()
        .scan(Position::fresh(&map), |position, path_part| {
            position.apply(*path_part, &map, &teleport_map);
            Some(*position)
        })
        .collect::<Vec<_>>();

    map.print(Some(&positions));

    let last = positions.last().unwrap();
    {
        use Direction::{D, L, R, U};

        Ok(((last.y + 1) * 1000
            + (last.x + 1) * 4
            + match last.d {
                R => 0,
                D => 1,
                L => 2,
                U => 3,
            })
        .to_string())
    }
}

pub(crate) const DAY: Day = Day {
    number: 22,
    part1: solve1,
    part2: solve2,
};
