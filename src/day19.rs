use std::ops::{Index, IndexMut};

use nom::{
    bytes::complete::tag,
    combinator::map,
    sequence::{preceded, terminated, tuple},
    IResult,
};

use crate::daylib::Day;
use crate::shared::parse_unum;

// obs is short for Obsidian
#[derive(Debug, Copy, Clone)]
struct Blueprint {
    id: usize,
    ore: PerResource,
    clay: PerResource,
    obsidian: PerResource,
    geode: PerResource,
}

impl Index<Resource> for Blueprint {
    type Output = PerResource;

    fn index(&self, index: Resource) -> &Self::Output {
        use Resource::{Clay, Geode, Obsidian, Ore};

        match index {
            Ore => &self.ore,
            Clay => &self.clay,
            Obsidian => &self.obsidian,
            Geode => &self.geode,
        }
    }
}

impl Blueprint {
    fn parse(i: &str) -> IResult<&str, Self> {
        map(
            terminated(
                tuple((
                    preceded(tag("Blueprint "), parse_unum::<usize>),
                    preceded(tag(": Each ore robot costs "), parse_unum::<usize>),
                    preceded(tag(" ore. Each clay robot costs "), parse_unum::<usize>),
                    preceded(tag(" ore. Each obsidian robot costs "), parse_unum::<usize>),
                    preceded(tag(" ore and "), parse_unum::<usize>),
                    preceded(tag(" clay. Each geode robot costs "), parse_unum::<usize>),
                    preceded(tag(" ore and "), parse_unum::<usize>),
                )),
                tag(" obsidian."),
            ),
            |(id, ore_ore, clay_ore, obs_ore, obs_clay, geode_ore, geode_obs)| {
                let zero = PerResource::default();
                Blueprint {
                    id,
                    ore: PerResource {
                        ore: ore_ore,
                        ..zero
                    },
                    clay: PerResource {
                        ore: clay_ore,
                        ..zero
                    },
                    obsidian: PerResource {
                        ore: obs_ore,
                        clay: obs_clay,
                        ..zero
                    },
                    geode: PerResource {
                        ore: geode_ore,
                        obsidian: geode_obs,
                        ..zero
                    },
                }
            },
        )(i)
    }
}

#[derive(Debug, Clone, Copy)]
enum Resource {
    Ore,
    Clay,
    Obsidian,
    Geode,
}

#[derive(Debug, Default, Clone, Copy)]
struct PerResource {
    ore: usize,
    clay: usize,
    obsidian: usize,
    geode: usize,
}

impl Index<Resource> for PerResource {
    type Output = usize;

    fn index(&self, index: Resource) -> &Self::Output {
        use Resource::{Clay, Geode, Obsidian, Ore};
        match index {
            Ore => &self.ore,
            Clay => &self.clay,
            Obsidian => &self.obsidian,
            Geode => &self.geode,
        }
    }
}

impl IndexMut<Resource> for PerResource {
    fn index_mut(&mut self, index: Resource) -> &mut Self::Output {
        use Resource::{Clay, Geode, Obsidian, Ore};
        match index {
            Ore => &mut self.ore,
            Clay => &mut self.clay,
            Obsidian => &mut self.obsidian,
            Geode => &mut self.geode,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SearchNode {
    stock: PerResource,
    robots: PerResource,
    time_left: usize,
}

fn add_production(mut stock: PerResource, robots: PerResource) -> PerResource {
    use Resource::{Clay, Geode, Obsidian, Ore};

    for r in [Geode, Obsidian, Clay, Ore] {
        stock[r] += robots[r];
    }

    stock
}

fn subtract_robot_cost(
    mut stock: PerResource,
    blueprint: Blueprint,
    robot: Resource,
) -> PerResource {
    use Resource::{Clay, Geode, Obsidian, Ore};

    for r in [Geode, Obsidian, Clay, Ore] {
        stock[r] -= blueprint[robot][r];
    }

    stock
}

#[allow(dead_code)]
fn best_heuristic_loop(node: SearchNode, blueprint: Blueprint) -> usize {
    use Resource::{Geode, Obsidian};

    let mut geodes = node.stock[Geode];
    let mut geode_robots = node.robots[Geode];
    let mut obsidian_robots = node.robots[Obsidian];
    let mut obsidian = node.stock[Obsidian];
    for _ in 0..=node.time_left {
        obsidian += obsidian_robots;
        geodes += geode_robots;
        if obsidian >= blueprint[Geode][Obsidian] {
            obsidian = obsidian.checked_sub(blueprint[Geode][Obsidian]).unwrap();
            geode_robots += 1;
        }
        obsidian_robots += 1;
    }

    geodes
}

#[allow(dead_code)]
fn best_heuristic_nikita(node: SearchNode, blueprint: Blueprint) -> usize {
    use Resource::{Geode, Obsidian};

    let t = node.time_left;
    let future_obsidian = t * (t + 1) / 2;
    let obsidian_end = node.stock[Obsidian] + node.robots[Obsidian] * t + future_obsidian;
    node.stock[Geode] + node.robots[Geode] * t + (obsidian_end / blueprint[Geode][Obsidian]) * t
}

fn max_geodes(blueprint: Blueprint, time: usize) -> usize {
    use Resource::{Clay, Geode, Obsidian, Ore};

    const ALL_RESOURCES: [Resource; 4] = [Ore, Clay, Obsidian, Geode];

    let mut start_robots = PerResource::default();
    start_robots[Ore] += 1;

    let mut frontier = vec![SearchNode {
        stock: PerResource::default(),
        robots: start_robots,
        time_left: time,
    }];

    let mut most_geodes = 0;

    println!("{blueprint:?}");

    while let Some(node) = frontier.pop() {
        // if iterations % 1_000_000 == 0 {
        //     println!("{iterations} iterations, {trimmed} trimmed");
        //     println!("{node:?}");
        // }

        if node.stock[Geode] > most_geodes {
            most_geodes = node.stock[Geode];
            println!("new best {most_geodes}; {node:?}");
        }

        if node.time_left == 0 {
            continue;
        }

        if best_heuristic_loop(node, blueprint) <= most_geodes {
            continue;
        }

        frontier.push(SearchNode {
            stock: add_production(node.stock, node.robots),
            time_left: node.time_left - 1,
            ..node
        });

        ALL_RESOURCES
            .iter()
            .filter(|robot| {
                ALL_RESOURCES
                    .iter()
                    .all(|r| blueprint[**robot][*r] <= node.stock[*r])
            })
            .for_each(|robot| {
                let mut new_robots = node.robots;
                new_robots[*robot] += 1;

                frontier.push(SearchNode {
                    stock: (add_production(
                        subtract_robot_cost(node.stock, blueprint, *robot),
                        node.robots,
                    )),
                    robots: new_robots,
                    time_left: node.time_left - 1,
                });
            });
    }

    most_geodes
}

#[allow(clippy::unnecessary_wraps)]
fn solve1(input: &str) -> color_eyre::Result<String> {
    let blueprints = input
        .lines()
        .map(|l| Blueprint::parse(l).unwrap().1)
        .collect::<Vec<_>>();

    Ok(blueprints
        .iter()
        .map(|b| b.id * max_geodes(*b, 24))
        .sum::<usize>()
        .to_string())
}

#[allow(clippy::unnecessary_wraps)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    let blueprints = input
        .lines()
        .map(|l| Blueprint::parse(l).unwrap().1)
        .collect::<Vec<_>>();

    Ok(blueprints
        .iter()
        .take(3)
        .map(|b| max_geodes(*b, 32))
        .product::<usize>()
        .to_string())
}

pub(crate) const DAY: Day = Day {
    number: 19,
    part1: solve1,
    part2: solve2,
};
