use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;
use std::{cmp, fmt, vec};

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::alpha1,
    combinator::map,
    multi::separated_list1,
    sequence::{preceded, tuple},
    IResult,
};
use rayon::prelude::*;

use crate::daylib::Day;
use crate::shared::parse_unum;

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
struct ValveName([char; 2]);

impl ValveName {
    fn new(s: &str) -> ValveName {
        let mut it = s.chars();
        if let (Some(a), Some(b), None) = (it.next(), it.next(), it.next()) {
            ValveName([a, b])
        } else {
            panic!("can't produce ValveName from {s}");
        }
    }
}

impl fmt::Display for ValveName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.0.iter().collect::<String>())
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Valve {
    name: ValveName,
    flow: u32,
    tunnels: Vec<ValveName>,
}

impl Valve {
    fn parse(i: &str) -> IResult<&str, Self> {
        map(
            tuple((
                preceded(tag("Valve "), alpha1),
                preceded(tag(" has flow rate="), parse_unum::<u32>),
                preceded(
                    alt((
                        tag("; tunnels lead to valves "),
                        tag("; tunnel leads to valve "),
                    )),
                    separated_list1(tag(", "), alpha1),
                ),
            )),
            |(name, flow, tunnels)| Valve {
                name: ValveName::new(name),
                flow,
                tunnels: tunnels.into_iter().map(ValveName::new).collect(),
            },
        )(i)
    }
}

fn shortest_path(
    valve_index: &HashMap<ValveName, Valve>,
    from: ValveName,
    to: ValveName,
    seen: &mut HashSet<ValveName>,
) -> Option<u32> {
    let mut q = valve_index[&from]
        .tunnels
        .iter()
        .copied()
        .map(|name| (1, name))
        .collect::<VecDeque<_>>();

    while let Some((distance, name)) = q.pop_front() {
        if name == to {
            return Some(distance);
        }
        if seen.contains(&name) {
            continue;
        }

        seen.insert(name);
        for next_name in &valve_index[&name].tunnels {
            q.push_back((distance + 1, *next_name));
        }
    }

    None
}

fn search(
    valve_index: &HashMap<ValveName, Valve>,
    nonzero_valves: &[ValveName],
    shortest_paths: &HashMap<(ValveName, ValveName), u32>,
    start: ValveName,
    initial_time: u32,
) -> (u32, Vec<ValveName>) {
    #[derive(Debug, Clone)]
    struct Node {
        valve: ValveName,
        time_left: u32,
        release: u32,
        path: Vec<ValveName>,
        valves_left: Vec<ValveName>,
    }

    let mut frontier = vec![Node {
        valve: start,
        time_left: initial_time,
        release: 0,
        path: vec![],
        valves_left: nonzero_valves
            .iter()
            .filter(|n| **n != start)
            // we use presorting here to avoid re-sorting in the trimming heuristic below
            .sorted_by_key(|n| cmp::Reverse(valve_index[n].flow))
            .copied()
            .collect(),
    }];
    frontier.reserve(100);
    let mut best = frontier.last().unwrap().clone();

    //let mut bails = 0;
    //let mut nodes = 0;

    while let Some(node) = frontier.pop() {
        // println!(
        //     "{}standing at{} (will release {}, {} time left), arrived by {}",
        //     " ".repeat(node.time_left as usize),
        //     node.valve,
        //     node.release,
        //     node.time_left,
        //     Itertools::intersperse(node.path.iter().map(|x| format!("{x}")), "-".to_string())
        //         .collect::<String>()
        // );

        //nodes += 1;

        if node.release > best.release {
            best = node.clone();
        }

        let upper_bound = node.release
            + node
                .valves_left
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    // next valve is at least 2 time units away
                    // valves_left is sorted in descending flow order, which means we're
                    // as optimistic as possible
                    valve_index[n].flow
                        * (node
                            .time_left
                            .saturating_sub((1 + u32::try_from(i).unwrap()) * 2))
                })
                .sum::<u32>();

        if upper_bound < best.release {
            // println!(
            //     "{}current best release {}, remaining upper bound is {}, bailing out",
            //     " ".repeat(node.time_left as usize),
            //     best.release,
            //     upper_bound
            // );
            //bails += 1;
            continue;
        }

        frontier.extend(node.valves_left.iter().enumerate().filter_map(|(i, n)| {
            let shortest_path = shortest_paths[&(node.valve, *n)];
            if let Some(new_time_left) = node.time_left.checked_sub(shortest_path + 1) {
                let new_path = node.path.iter().chain([&node.valve]).copied().collect();

                let mut new_valves_left = Vec::with_capacity(node.valves_left.len() - 1);
                for (j, v) in node.valves_left.iter().enumerate() {
                    if j != i {
                        new_valves_left.push(*v);
                    }
                }

                Some(Node {
                    valve: *n,
                    time_left: new_time_left,
                    release: node.release + valve_index[n].flow * new_time_left,
                    path: new_path,
                    valves_left: new_valves_left,
                })
            } else {
                None
            }
        }));
    }

    //println!("{nodes} nodes, {bails} bails");

    let mut full_path = best.path.clone();
    full_path.push(best.valve);
    (best.release, full_path)
}

#[allow(clippy::unnecessary_wraps)]
fn solve1(input: &str) -> color_eyre::Result<String> {
    let all_valves = input
        .lines()
        .map(|l| Valve::parse(l).unwrap().1)
        .collect::<Vec<_>>();

    let valve_index = all_valves
        .iter()
        .map(|v| (v.name, (*v).clone()))
        .collect::<HashMap<_, Valve>>();

    let nonzero_valves = all_valves
        .iter()
        .filter(|v| v.flow > 0)
        .map(|v| v.name)
        .collect::<Vec<_>>();

    let shortest_paths = nonzero_valves
        .iter()
        // need to add AA back to find paths from the start node
        .chain(&[ValveName::new("AA")])
        .permutations(2)
        .flat_map(|perm| {
            let (va, vb) = (*perm[0], *perm[1]);
            let d = shortest_path(&valve_index, va, vb, &mut HashSet::new()).unwrap();
            vec![((va, vb), d), ((vb, va), d)].into_iter()
        })
        .collect::<HashMap<_, _>>();

    let now = Instant::now();

    let result = search(
        &valve_index,
        &nonzero_valves,
        &shortest_paths,
        ValveName::new("AA"),
        30,
    );

    println!("{}", now.elapsed().as_secs_f32());

    Ok(result.0.to_string())
}

#[allow(clippy::unnecessary_wraps)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    let all_valves = input
        .lines()
        .map(|l| Valve::parse(l).unwrap().1)
        .collect::<Vec<_>>();

    let valve_index = all_valves
        .iter()
        .map(|v| (v.name, (*v).clone()))
        .collect::<HashMap<_, Valve>>();

    let interesting_valves = all_valves
        .iter()
        .filter(|v| v.flow > 0)
        .map(|v| v.name)
        .collect::<Vec<_>>();

    let shortest_paths = interesting_valves
        .iter()
        // need to add AA back to find paths from the start node
        .chain(&[ValveName::new("AA")])
        .permutations(2)
        .flat_map(|perm| {
            let (va, vb) = (*perm[0], *perm[1]);
            let d = shortest_path(&valve_index, va, vb, &mut HashSet::new()).unwrap();
            vec![((va, vb), d), ((vb, va), d)].into_iter()
        })
        .collect::<HashMap<_, _>>();

    // let mut best_paths = None;
    let best_release = (1..=(interesting_valves.len() / 2 + 1))
        .flat_map(|n_mine| interesting_valves.iter().copied().combinations(n_mine))
        .par_bridge()
        .fold(
            || 0,
            |acc, my_valves| {
                let elephant_valves = interesting_valves
                    .iter()
                    .filter(|v| !my_valves.contains(v))
                    .copied()
                    .collect::<Vec<_>>();

                let (my_release, _my_path) = search(
                    &valve_index,
                    &my_valves,
                    &shortest_paths,
                    ValveName::new("AA"),
                    26,
                );
                let (elephant_release, _elephant_path) = search(
                    &valve_index,
                    &elephant_valves,
                    &shortest_paths,
                    ValveName::new("AA"),
                    26,
                );

                acc.max(my_release + elephant_release)
            },
        )
        .max()
        .unwrap();

    Ok((best_release).to_string())
}

pub(crate) const DAY: Day = Day {
    number: 16,
    part1: solve1,
    part2: solve2,
};
