use std::str;

use color_eyre::eyre::eyre;

use crate::daylib::Day;

fn to_idx(c: u8) -> usize {
    (c - b'a') as usize
}

// fn print_state(i: usize, window: &[u8]) {
//     println!("{}: {}", i, str::from_utf8(window).unwrap());
// }

fn find_marker(s: &str, len: usize) -> Option<usize> {
    let mut counts = [false; 26];
    let mut last: Option<u8> = None;
    for (i, window) in s.as_bytes().windows(len).enumerate() {
        if let Some(out) = last {
            counts[to_idx(out)] &= false;
        }
        for c in window {
            counts[to_idx(*c)] |= true;
        }
        if counts.iter().filter(|x| **x).count() == len {
            return Some(i + len);
        }
        last = Some(*(window.first().unwrap()));
    }
    None
}

fn solve1(input: &str) -> color_eyre::Result<String> {
    match find_marker(input, 4) {
        Some(n) => Ok(n.to_string()),
        None => Err(eyre!("didn't find unique window of length 4 in the input")),
    }
}

fn solve2(input: &str) -> color_eyre::Result<String> {
    match find_marker(input, 14) {
        Some(n) => Ok(n.to_string()),
        None => Err(eyre!("didn't find unique window of length 4 in the input")),
    }
}

pub(crate) const DAY: Day = Day {
    number: 6,
    part1: solve1,
    part2: solve2,
};
