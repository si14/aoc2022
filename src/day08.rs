use std::fmt;
use std::str::FromStr;

use crate::daylib::Day;

struct TreeMap<T: Copy> {
    w: usize,
    h: usize,
    d: Vec<T>,
}

impl<T> FromStr for TreeMap<T>
where
    T: Copy + FromStr,
    <T as FromStr>::Err: fmt::Debug,
{
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let w = s.lines().next().unwrap().len();
        let h = s.lines().count();
        let d = s
            .lines()
            .flat_map(str::chars)
            .map(|c| c.to_string().parse().unwrap())
            .collect();

        Ok(TreeMap { w, h, d })
    }
}

impl<T: Copy + fmt::Debug> TreeMap<T> {
    fn blank(w: usize, h: usize, default: T) -> TreeMap<T> {
        TreeMap {
            w,
            h,
            d: vec![default; w * h],
        }
    }

    fn slicing_iter(&self, direction: Direction, slice: usize) -> TreeMapSlicingIter<'_, T> {
        assert!(slice < direction.limit(self));
        TreeMapSlicingIter {
            tree_map: self,
            direction,
            slice,
            i: 0,
        }
    }

    fn at(&self, x: usize, y: usize) -> T {
        self.d[x + y * self.w]
    }
}

#[test]
fn tree_map_at_test() {
    // 0 1
    // 2 3
    // 4 5
    let tree_map = TreeMap {
        w: 2,
        h: 3,
        d: vec![0, 1, 2, 3, 4, 5],
    };
    assert_eq!(tree_map.at(0, 1), 2);
    assert_eq!(tree_map.at(1, 1), 3);
    assert_eq!(tree_map.at(0, 2), 4);
}

impl<T: Copy + fmt::Debug> fmt::Debug for TreeMap<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dir = Direction::LeftRight;
        for slice in 0..dir.slice_limit(self) {
            for x in self.slicing_iter(dir, slice) {
                write!(f, "{x:?}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    TopBottom,
    BottomTop,
    LeftRight,
    RightLeft,
}

impl Direction {
    // non-inclusive
    fn limit<T: Copy>(self, tree_map: &TreeMap<T>) -> usize {
        use Direction::{BottomTop, LeftRight, RightLeft, TopBottom};

        match self {
            TopBottom | BottomTop => tree_map.h,
            LeftRight | RightLeft => tree_map.w,
        }
    }

    fn slice_limit<T: Copy>(self, tree_map: &TreeMap<T>) -> usize {
        use Direction::{BottomTop, LeftRight, RightLeft, TopBottom};

        match self {
            TopBottom | BottomTop => tree_map.w,
            LeftRight | RightLeft => tree_map.h,
        }
    }

    // slice is counted from top/left corner
    fn idx<T: Copy>(self, tree_map: &TreeMap<T>, slice: usize, i: usize) -> usize {
        use Direction::{BottomTop, LeftRight, RightLeft, TopBottom};

        match self {
            TopBottom | BottomTop => {
                assert!(slice < tree_map.w);
                assert!(i < tree_map.h);
            }
            LeftRight | RightLeft => {
                assert!(slice < tree_map.h);
                assert!(i < tree_map.w);
            }
        }

        assert!(i <= self.limit(tree_map));

        match self {
            LeftRight => slice * tree_map.w + i,
            RightLeft => slice * tree_map.w + (tree_map.w - 1 - i),
            TopBottom => slice + i * tree_map.w,
            BottomTop => slice + (tree_map.h - 1 - i) * tree_map.w,
        }
    }
}

#[cfg(test)]
mod direction_tests {
    use test_case::test_case;

    use crate::day08::{
        Direction,
        Direction::{BottomTop, LeftRight, RightLeft, TopBottom},
        TreeMap,
    };

    #[test_case(LeftRight, 0, 0 => 0)]
    #[test_case(RightLeft, 0, 0 => 1)]
    #[test_case(TopBottom, 0, 0 => 0)]
    #[test_case(BottomTop, 0, 0 => 4)]
    #[test_case(LeftRight, 1, 0 => 2)]
    #[test_case(RightLeft, 1, 0 => 3)]
    #[test_case(TopBottom, 1, 0 => 1)]
    #[test_case(BottomTop, 1, 0 => 5)]
    #[test_case(LeftRight, 2, 1 => 5)]
    #[test_case(RightLeft, 2, 1 => 4)]
    #[test_case(TopBottom, 1, 2 => 5)]
    #[test_case(BottomTop, 1, 2 => 1)]
    fn to_idx_tests(d: Direction, slice: usize, i: usize) -> usize {
        // 0 1
        // 2 3
        // 4 5
        let tree_map = TreeMap {
            w: 2,
            h: 3,
            d: vec![0, 1, 2, 3, 4, 5],
        };

        d.idx(&tree_map, slice, i)
    }
}

struct TreeMapSlicingIter<'a, T: Copy> {
    tree_map: &'a TreeMap<T>,
    direction: Direction,
    slice: usize,
    i: usize,
}

impl<'a, T: Copy> Iterator for TreeMapSlicingIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let TreeMapSlicingIter {
            tree_map,
            direction,
            slice,
            i,
        } = self;

        let limit = direction.limit(tree_map);

        if *i < limit {
            *i += 1;
            Some(tree_map.d[direction.idx(tree_map, *slice, (*i) - 1)])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod iter_tests {
    use test_case::test_case;

    use crate::day08::{
        Direction,
        Direction::{BottomTop, LeftRight},
        TreeMap,
    };

    #[test_case(LeftRight, 1 => vec![2, 3])]
    #[test_case(BottomTop, 1 => vec![5, 3, 1])]
    fn iter_tests(d: Direction, slice: usize) -> Vec<u8> {
        // 0 1
        // 2 3
        // 4 5
        let tree_map = TreeMap {
            w: 2,
            h: 3,
            d: vec![0, 1, 2, 3, 4, 5],
        };

        tree_map.slicing_iter(d, slice).collect()
    }
}

fn solve1(input: &str) -> color_eyre::Result<String> {
    use crate::day08::Direction::{BottomTop, LeftRight, RightLeft, TopBottom};

    let tree_map = TreeMap::<u8>::from_str(input)?;
    let mut visible = TreeMap::blank(tree_map.w, tree_map.h, 0u8);

    for dim in &vec![LeftRight, RightLeft, TopBottom, BottomTop] {
        for slice in 0..dim.limit(&tree_map) {
            tree_map
                .slicing_iter(*dim, slice)
                .enumerate()
                .fold(None, |max_seen, (i, height)| {
                    if max_seen.is_none() || max_seen.unwrap() < height {
                        let idx = dim.idx(&visible, slice, i);
                        visible.d[idx] |= 1;
                        Some(height)
                    } else {
                        max_seen
                    }
                });
        }
    }

    Ok(visible.d.iter().filter(|x| **x > 0).count().to_string())
}

fn visibility_score(tm: &TreeMap<u8>, x: usize, y: usize) -> usize {
    if x == 0 || y == 0 || tm.w - x == 1 || tm.h - y == 1 {
        return 0;
    }

    let current = tm.at(x, y);
    let right = (x..tm.w)
        .map(|i| tm.at(i, y))
        .enumerate()
        .find(|(idx, height)| *idx != 0 && height >= &current)
        // seeing to the edge
        .unwrap_or(((tm.w - x - 1), 0))
        .0;
    let left = (0..=x)
        .rev()
        .map(|i| tm.at(i, y))
        .enumerate()
        .find(|(idx, height)| *idx != 0 && height >= &current)
        .unwrap_or((x, 0))
        .0;
    let down = (y..tm.h)
        .map(|i| tm.at(x, i))
        .enumerate()
        .find(|(idx, height)| *idx != 0 && height >= &current)
        .unwrap_or(((tm.h - y - 1), 0))
        .0;
    let up = (0..=y)
        .rev()
        .map(|i| tm.at(x, i))
        .enumerate()
        .find(|(idx, height)| *idx != 0 && height >= &current)
        .unwrap_or((y, 0))
        .0;

    up * down * left * right
}

fn solve2(input: &str) -> color_eyre::Result<String> {
    let tree_map = TreeMap::<u8>::from_str(input)?;
    Ok((0..tree_map.h)
        .flat_map(|y| (0..tree_map.w).map(move |x| (x, y)))
        .map(|(x, y)| visibility_score(&tree_map, x, y))
        .max()
        .unwrap()
        .to_string())
}

pub(crate) const DAY: Day = Day {
    number: 8,
    part1: solve1,
    part2: solve2,
};
