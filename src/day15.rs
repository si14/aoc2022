use std::collections::HashSet;

use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    combinator::map,
    sequence::{preceded, tuple},
    IResult,
};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::SmallRng;
use rand::SeedableRng;

use crate::daylib::Day;
use crate::shared::parse_inum;

#[derive(Debug)]
struct BeaconReport {
    sensor: (i64, i64),
    beacon: (i64, i64),
    r: u64,
}

impl BeaconReport {
    fn new(sensor @ (sx, sy): (i64, i64), beacon @ (bx, by): (i64, i64)) -> Self {
        let r = sx.abs_diff(bx) + sy.abs_diff(by);
        Self { sensor, beacon, r }
    }

    fn parse(i: &str) -> IResult<&str, Self> {
        map(
            tuple((
                preceded(tag("Sensor at x="), parse_inum::<i64>),
                preceded(tag(", y="), parse_inum::<i64>),
                preceded(tag(": closest beacon is at x="), parse_inum::<i64>),
                preceded(tag(", y="), parse_inum::<i64>),
            )),
            |(sx, sy, bx, by)| BeaconReport::new((sx, sy), (bx, by)),
        )(i)
    }

    fn in_range(&self, (x, y): (i64, i64)) -> bool {
        self.sensor.0.abs_diff(x) + self.sensor.1.abs_diff(y) <= self.r
    }
}

fn solve1(input: &str) -> color_eyre::Result<String> {
    let mut lines = input.lines();
    let test_y: i64 = lines.next().unwrap().parse()?;
    // ignore explicit search space line
    let _ = lines.next();

    let relevant_reports = lines
        .map(|l| BeaconReport::parse(l).unwrap().1)
        .filter(|br| br.sensor.1.abs_diff(test_y) <= br.r)
        .collect::<Vec<_>>();

    let min_x = relevant_reports
        .iter()
        .map(|br| br.sensor.0 - i64::try_from(br.r).unwrap() - 1)
        .min()
        .unwrap();
    let max_x = relevant_reports
        .iter()
        .map(|br| br.sensor.0 + i64::try_from(br.r).unwrap() + 1)
        .max()
        .unwrap();

    let num_beacons_on_test_y = relevant_reports
        .iter()
        .filter(|br| br.beacon.1 == test_y)
        .map(|br| br.beacon.0)
        .unique()
        .count();

    let xs_in_range = (min_x..=max_x)
        .filter(|x| relevant_reports.iter().any(|br| br.in_range((*x, test_y))))
        .count();

    Ok((xs_in_range - num_beacons_on_test_y).to_string())
}

fn repulsion((x, y): (i64, i64), reports: &[BeaconReport]) -> i64 {
    reports
        .iter()
        .map(|br| {
            let distance = br.sensor.0.abs_diff(x) + br.sensor.1.abs_diff(y);
            if distance > br.r {
                0i64
            } else {
                // add +1 to account for distance = r (should still repulse)
                i64::try_from(br.r.checked_sub(distance).unwrap()).unwrap() + 1
            }
        })
        .sum()
}

fn better_neighbour(
    now @ (x, y): (i64, i64),
    search_space: i64,
    reports: &[BeaconReport],
) -> Option<(i64, i64)> {
    let repulsion_now = repulsion(now, reports);

    vec![(0, 1), (0, -1), (1, 0), (-1, 0)]
        .iter()
        .map(|(dx, dy)| (x + dx, y + dy))
        .filter(|(x, y)| *x >= 0 && *x <= search_space && *y >= 0 && *y <= search_space)
        .map(|point| (repulsion(point, reports), point))
        .filter(|(repulsion, _coords)| *repulsion < repulsion_now)
        .sorted()
        .next()
        .map(|(_repulsion, coords)| coords)
}

// doesn't work, local minima defeat it :(
#[allow(dead_code)]
fn solve2_localsearch(input: &str) -> color_eyre::Result<String> {
    let mut lines = input.lines();
    // ignore the test line
    let _ = lines.next();
    let search_space: i64 = lines.next().unwrap().parse()?;

    let reports = lines
        .map(|l| BeaconReport::parse(l).unwrap().1)
        .collect::<Vec<_>>();

    let known_beacons = reports.iter().map(|br| br.beacon).collect::<HashSet<_>>();

    let mut rng = SmallRng::seed_from_u64(42 + 2);
    let sampler = Uniform::from(1..=search_space);
    let mut loops = 0;
    let hidden_beacon;

    loop {
        let mut test_point = (sampler.sample(&mut rng), sampler.sample(&mut rng));
        while let Some(next_point) = better_neighbour(test_point, search_space, &reports) {
            test_point = next_point;
        }
        let final_repulsion = repulsion(test_point, &reports);
        if final_repulsion > 0 {
            println!("[{loops}] found a local minimum at {test_point:?} ({final_repulsion})");
            loops += 1;
            continue;
        }
        if known_beacons.contains(&test_point) {
            println!("[{loops}] found a known beacon at {test_point:?}");
            loops += 1;
            continue;
        }

        println!("[{loops}] found the hidden beacon at {test_point:?}!");
        hidden_beacon = test_point;
        break;
    }

    Ok((hidden_beacon.0 * 4_000_000 + hidden_beacon.1).to_string())
}

fn solve2(input: &str) -> color_eyre::Result<String> {
    let mut lines = input.lines();
    // ignore the test line
    let _ = lines.next();
    let search_space: i64 = lines.next().unwrap().parse()?;

    let reports = lines
        .map(|l| BeaconReport::parse(l).unwrap().1)
        .collect::<Vec<_>>();

    let mut result = None;

    let sides = vec![(0, -1), (0, 1), (-1, 0), (1, 0)];
    for (
        _i,
        BeaconReport {
            beacon: _,
            sensor,
            r,
        },
    ) in reports.iter().enumerate()
    {
        for offset in 0..=(r + 1) {
            for (a, b) in &sides {
                let r = i64::try_from(*r).unwrap();
                let offset = i64::try_from(offset).unwrap();

                let dx = if *a == 0 {
                    offset
                } else {
                    ((r + 1) - offset) * a
                };
                let dy = if *b == 0 {
                    offset
                } else {
                    ((r + 1) - offset) * b
                };

                let x = sensor.0 + dx;
                let y = sensor.1 + dy;

                if !(x >= 0 && x <= search_space && y >= 0 && y <= search_space) {
                    continue;
                }

                if !reports.iter().any(|r| r.in_range((x, y))) {
                    result = Some((x, y));
                }
            }
        }
    }

    let result = result.unwrap();

    Ok((result.0 * 4_000_000 + result.1).to_string())
}

pub(crate) const DAY: Day = Day {
    number: 15,
    part1: solve1,
    part2: solve2,
};
