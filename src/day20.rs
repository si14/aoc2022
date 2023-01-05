use std::collections::VecDeque;

use crate::daylib::Day;

#[derive(Debug, Copy, Clone)]
struct Num(usize, i32);

// it's a bit sloppy with casts but w/e
#[allow(
    clippy::unnecessary_wraps,
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn solve1(input: &str) -> color_eyre::Result<String> {
    let original = input
        .lines()
        .map(|l| l.parse::<i32>().unwrap())
        .collect::<Vec<_>>();
    let n = original.len();
    let mut mixed = original
        .iter()
        .enumerate()
        .map(|(i, x)| Num(i, *x))
        .collect::<VecDeque<_>>();

    for (i, shift) in original.into_iter().enumerate() {
        let j = mixed
            .iter()
            .position(|Num(orig_idx, _)| *orig_idx == i)
            .unwrap();

        //println!("{:?} should shift {shift}", mixed[j].1);
        //println!("{:?}", mixed.iter().map(|x| x.1).collect::<Vec<_>>());
        if shift >= 0 {
            mixed.rotate_left(j);
            let x = mixed.pop_front().unwrap();
            mixed.insert((shift % (n - 1) as i32) as usize, x);
        } else {
            mixed.rotate_right(n - 1 - j);
            let x = mixed.pop_back().unwrap();
            mixed.insert((n as i32 - 1 + (shift % (n - 1) as i32)) as usize, x);
        }
        //println!("->\n{:?}", mixed.iter().map(|x| x.1).collect::<Vec<_>>());
    }

    //println!("{mixed:?}");

    mixed.rotate_left(mixed.iter().position(|Num(_, x)| *x == 0).unwrap());

    // println!(
    //     "{:?}",
    //     (mixed[1000 % n].1, mixed[2000 % n].1, mixed[3000 % n].1)
    // );

    Ok((mixed[1000 % n].1 + mixed[2000 % n].1 + mixed[3000 % n].1).to_string())
}

#[derive(Debug, Copy, Clone)]
struct Num2 {
    orig_idx: usize,
    x: i64,
}

#[allow(
    clippy::unnecessary_wraps,
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    let original = input
        .lines()
        .map(|l| l.parse::<i32>().unwrap())
        .collect::<Vec<_>>();
    let n = original.len();
    let mut mixed = original
        .iter()
        .enumerate()
        .map(|(i, x)| Num2 {
            orig_idx: i,
            x: *x as i64 * 811_589_153,
        })
        .collect::<VecDeque<_>>();

    for _ in 0..10 {
        for (i, shift) in original.iter().enumerate() {
            let shift = (*shift as i64 * 811_589_153 % (n as i64 - 1)) as i32;

            let j = mixed
                .iter()
                .position(|Num2 { orig_idx, .. }| *orig_idx == i)
                .unwrap();

            //println!("{:?} should shift {shift}", mixed[j].1);
            //println!("{:?}", mixed.iter().map(|x| x.full_x).collect::<Vec<_>>());
            if shift >= 0 {
                mixed.rotate_left(j);
                let x = mixed.pop_front().unwrap();
                mixed.insert((shift % (n - 1) as i32) as usize, x);
            } else {
                mixed.rotate_right(n - 1 - j);
                let x = mixed.pop_back().unwrap();
                mixed.insert((n as i32 - 1 + (shift % (n - 1) as i32)) as usize, x);
            }
            //println!("->\n{:?}", mixed.iter().map(|x| x.1).collect::<Vec<_>>());
        }
    }

    //println!("{mixed:?}");

    mixed.rotate_left(
        mixed
            .iter()
            .position(|Num2 { x: full_x, .. }| *full_x == 0)
            .unwrap(),
    );

    // println!(
    //     "{:?}",
    //     (mixed[1000 % n].1, mixed[2000 % n].1, mixed[3000 % n].1)
    // );

    Ok((mixed[1000 % n].x + mixed[2000 % n].x + mixed[3000 % n].x).to_string())
}

pub(crate) const DAY: Day = Day {
    number: 20,
    part1: solve1,
    part2: solve2,
};
