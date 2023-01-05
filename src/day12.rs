use binary_heap_plus as bhp;
use itertools::Itertools;

use crate::daylib::Day;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum MapPoint {
    Start,
    End,
    Point(u8),
}

impl MapPoint {
    fn elevation(self) -> u8 {
        match self {
            Self::Start => b'a',
            Self::End => b'z',
            Self::Point(x) => x,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct RowCol(usize, usize);

impl RowCol {
    fn row(&self) -> usize {
        self.0
    }

    fn col(&self) -> usize {
        self.1
    }
}

// 0;0 is top left
#[derive(Debug, Clone)]
struct Map {
    data: Vec<MapPoint>,
    start: RowCol,
    end: RowCol,
    size: RowCol,
}

impl Map {
    fn parse(input: &str) -> Self {
        use MapPoint::{End, Point, Start};

        let cols = input.lines().next().unwrap().chars().count();
        let rows = input.lines().count();
        let mut start = None;
        let mut end = None;
        Self {
            data: input
                .chars()
                .filter(|c| *c != '\n' && *c != '\r')
                .enumerate()
                .map(|(i, c)| match c {
                    'S' => {
                        start = Some(RowCol(i / cols, i % cols));
                        Start
                    }
                    'E' => {
                        end = Some(RowCol(i / cols, i % cols));
                        End
                    }
                    other => Point(other as u8),
                })
                .collect(),
            start: start.unwrap(),
            end: end.unwrap(),
            size: RowCol(rows, cols),
        }
    }

    #[allow(dead_code)]
    fn string_map(&self) -> String {
        Itertools::intersperse(
            self.data.chunks(self.size.col()).map(|row| {
                row.iter()
                    .map(|p| match p {
                        MapPoint::Start => 'S',
                        MapPoint::End => 'E',
                        MapPoint::Point(p) => *p as char,
                    })
                    .collect::<String>()
            }),
            "\n".to_string(),
        )
        .collect::<String>()
    }

    fn idx(&self, x: RowCol) -> usize {
        x.0 * self.size.col() + x.1
    }

    fn elevation(&self, x: RowCol) -> u8 {
        self.data[self.idx(x)].elevation()
    }

    #[allow(clippy::cast_lossless)]
    fn neighbours(&self, x: RowCol) -> Vec<RowCol> {
        let up = if x.row() > 0 {
            Some(RowCol(x.0 - 1, x.1))
        } else {
            None
        };
        let down = if x.row() < (self.size.row() - 1) {
            Some(RowCol(x.0 + 1, x.1))
        } else {
            None
        };
        let left = if x.col() > 0 {
            Some(RowCol(x.0, x.1 - 1))
        } else {
            None
        };
        let right = if x.col() < (self.size.col() - 1) {
            Some(RowCol(x.0, x.1 + 1))
        } else {
            None
        };
        let x_elevation = self.elevation(x);

        vec![up, down, left, right]
            .iter()
            .flatten()
            .copied()
            // can get negative if we're higher than the heighbour and that's OK
            .filter(|p| (self.elevation(*p) as i16) - (x_elevation as i16) <= 1)
            .collect()
    }
}

#[derive(Debug)]
struct SearchNode {
    pos: RowCol,
    path_len: u32,
    full_path_guess: u32,
}

fn h(map: &Map, pos: RowCol) -> u32 {
    u32::try_from(map.end.0.abs_diff(pos.0) + map.end.1.abs_diff(pos.1)).unwrap()
}

fn a_star(map: &Map) -> (Vec<Option<u32>>, Vec<Option<RowCol>>) {
    // shortest known path from the start
    let mut min_path: Vec<Option<u32>> = vec![None; map.data.len()];
    min_path[map.idx(map.start)] = Some(0);

    // where did we get to it from
    let mut min_path_from: Vec<Option<RowCol>> = vec![None; map.data.len()];
    min_path_from[map.idx(map.start)] = None;

    let mut frontier = bhp::BinaryHeap::new_by(|a: &SearchNode, b: &SearchNode| {
        a.full_path_guess.cmp(&b.full_path_guess).reverse()
    });

    let start_node = SearchNode {
        pos: map.start,
        path_len: 0,
        full_path_guess: h(map, map.start),
    };
    frontier.push(start_node);

    while !frontier.is_empty() {
        let node = frontier.pop().unwrap();

        if node.pos == map.end {
            break;
        }

        for nei_pos in map.neighbours(node.pos) {
            let nei_idx = map.idx(nei_pos);
            let new_path = node.path_len + 1;
            let old_path = min_path[nei_idx];
            if old_path.is_none() || new_path < old_path.unwrap() {
                min_path[nei_idx] = Some(new_path);
                min_path_from[nei_idx] = Some(node.pos);
                frontier.push(SearchNode {
                    pos: nei_pos,
                    path_len: new_path,
                    full_path_guess: new_path + h(map, nei_pos),
                });
            }
        }
    }

    // println!(
    //     "{}",
    //     Itertools::intersperse(
    //         visited.chunks(map.size.col()).map(|line| line
    //             .iter()
    //             .map(|x| format!("{:?}\t", x))
    //             .collect::<String>()),
    //         "\n".to_string()
    //     )
    //     .collect::<String>()
    // );

    (min_path, min_path_from)
}

#[allow(dead_code)]
fn print_move_map(map: &Map, min_path_from: &[Option<RowCol>]) {
    let mut move_map = vec!['.'; map.data.len()];
    move_map[map.idx(map.end)] = 'E';
    let mut pos = map.end;
    while pos != map.start {
        let from_pos = min_path_from[map.idx(pos)]
            .unwrap_or_else(|| panic!("expected to find path to {pos:?}"));
        // there's a bug in idea rust plugin that makes "other" seem unused
        #[allow(unused_variables)]
        match (
            0isize
                .checked_add_unsigned(pos.row())
                .unwrap()
                .checked_sub_unsigned(from_pos.row())
                .unwrap(),
            0isize
                .checked_add_unsigned(pos.col())
                .unwrap()
                .checked_sub_unsigned(from_pos.col())
                .unwrap(),
        ) {
            (0, 1) => move_map[map.idx(from_pos)] = '>',
            (0, -1) => move_map[map.idx(from_pos)] = '<',
            (1, 0) => move_map[map.idx(from_pos)] = 'v',
            (-1, 0) => move_map[map.idx(from_pos)] = '^',
            other => panic!("unexpected move {other:?}"),
        }
        pos = from_pos;
    }
    println!(
        "{}",
        Itertools::intersperse(
            move_map
                .chunks(map.size.col())
                .map(|line| line.iter().collect::<String>()),
            "\n".to_string()
        )
        .collect::<String>()
    );
}

#[allow(clippy::unnecessary_wraps)]
fn solve1(input: &str) -> color_eyre::Result<String> {
    let map = Map::parse(input);
    let (min_path, _min_path_from) = a_star(&map);

    Ok(min_path[map.idx(map.end)].unwrap().to_string())
}

#[allow(clippy::unnecessary_wraps)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    let map = Map::parse(input);

    Ok(map
        .data
        .iter()
        .enumerate()
        .filter_map(|(i, mp)| {
            if mp.elevation() == b'a' {
                Some(i)
            } else {
                None
            }
        })
        .filter_map(|new_start_idx| {
            let mut map = map.clone();
            let old_start_idx = map.idx(map.start);
            map.data[old_start_idx] = MapPoint::Point(b'a');
            map.data[new_start_idx] = MapPoint::Start;
            map.start = RowCol(
                new_start_idx / map.size.col(),
                new_start_idx % map.size.col(),
            );
            let (min_path, _min_path_from) = a_star(&map);
            min_path[map.idx(map.end)]
        })
        .min()
        .unwrap()
        .to_string())
}

pub(crate) const DAY: Day = Day {
    number: 12,
    part1: solve1,
    part2: solve2,
};
