use std::cmp;
use std::collections::HashMap;

use color_eyre::eyre::eyre;
use nom::branch::alt;
use nom::character::complete::alpha1;
use nom::combinator::value;
use nom::sequence::separated_pair;
use nom::{bytes::complete::tag, combinator::map, sequence::tuple, IResult};

use crate::daylib::Day;
use crate::shared::parse_inum;

type Int = i64;

#[derive(Debug, Clone, Copy)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
}

impl Op {
    fn parse(i: &str) -> IResult<&str, Self> {
        use Op::{Add, Div, Mul, Sub};

        alt((
            value(Add, tag(" + ")),
            value(Sub, tag(" - ")),
            value(Mul, tag(" * ")),
            value(Div, tag(" / ")),
        ))(i)
    }

    fn apply(self, a: Int, b: Int) -> color_eyre::Result<Int> {
        use Op::{Add, Div, Eq, Mul, Sub};

        match self {
            Add => Ok(a + b),
            Sub | Eq => Ok(a - b),
            Mul => Ok(a * b),
            Div => {
                if a % b == 0 {
                    Ok(a / b)
                } else {
                    Err(eyre!("non-integer division"))
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Value {
    Calc(String, Op, String),
    Concrete(Int),
}

fn parse_name(i: &str) -> IResult<&str, String> {
    map(alpha1, str::to_string)(i)
}

impl Value {
    fn parse(i: &str) -> IResult<&str, Self> {
        use Value::{Calc, Concrete};

        alt((
            map(parse_inum::<Int>, Concrete),
            map(
                tuple((parse_name, Op::parse, parse_name)),
                |(n1, op, n2)| Calc(n1, op, n2),
            ),
        ))(i)
    }
}

#[derive(Debug, Clone)]
struct Monkey {
    name: String,
    value: Value,
}

impl Monkey {
    fn parse(i: &str) -> IResult<&str, Self> {
        map(
            separated_pair(parse_name, tag(": "), Value::parse),
            |(n, s)| Monkey { name: n, value: s },
        )(i)
    }
}

fn eval(memory: &mut HashMap<String, Monkey>) -> color_eyre::Result<()> {
    use Value::{Calc, Concrete};

    let mut stack = vec![memory["root"].clone()];
    while let Some(monkey) = stack.pop() {
        match monkey.value {
            Concrete(_) => continue,
            Calc(ref pointer_a, op, ref pointer_b) => {
                if let Concrete(a) = memory[pointer_a].value {
                    if let Concrete(b) = memory[pointer_b].value {
                        memory.insert(
                            monkey.name.clone(),
                            Monkey {
                                value: Concrete(op.apply(a, b)?),
                                name: monkey.name.clone(),
                            },
                        );
                    } else {
                        stack.push(monkey.clone());
                        stack.push(memory[pointer_b].clone());
                    }
                } else {
                    stack.push(monkey.clone());
                    stack.push(memory[pointer_a].clone());
                }
            }
        }
    }

    Ok(())
}

fn solve1(input: &str) -> color_eyre::Result<String> {
    let mut memory = input
        .lines()
        .map(|l| Monkey::parse(l).unwrap().1)
        .map(|m| (m.name.clone(), m))
        .collect::<HashMap<_, _>>();

    eval(&mut memory).expect("part1 doesn't involve non-integer division");

    if let Value::Concrete(x) = memory["root"].value {
        Ok(x.to_string())
    } else {
        panic!("unexpected root monkey value {:?}", memory["root"])
    }
}

#[allow(clippy::unnecessary_wraps)]
fn solve2(input: &str) -> color_eyre::Result<String> {
    use Value::{Calc, Concrete};

    let mut memory = input
        .lines()
        .map(|l| Monkey::parse(l).unwrap().1)
        .map(|m| (m.name.clone(), m))
        .collect::<HashMap<_, _>>();

    if let Monkey {
        name,
        value: Calc(a, _, b),
    } = &memory["root"]
    {
        memory.insert(
            "root".to_string(),
            Monkey {
                name: name.clone(),
                value: Calc(a.clone(), Op::Eq, b.clone()),
            },
        );
    } else {
        unreachable!();
    }

    let mut guess: Int = 0;
    let mut step: Int = 1;
    let mut last_distance: Option<Int> = None;
    let mut found = false;
    let result: Int;
    loop {
        //println!("guess: {guess}");
        let mut memory = memory.clone();

        memory.insert(
            "humn".to_string(),
            Monkey {
                name: "humn".to_string(),
                value: Concrete(guess),
            },
        );

        if eval(&mut memory).is_err() {
            // got non-integer division, bailing out and guessing elsewhere
            guess += step;
            continue;
        }

        if step == 0 {
            result = 42;
            break;
        }

        if let Concrete(guess_result) = memory["root"].value {
            println!(
                "new distance: {}, last distance: {}",
                guess_result.abs(),
                last_distance.unwrap_or(0)
            );
            if guess_result == 0 {
                result = guess;
                break;
            }

            let new_distance = guess_result.abs();

            if let Some(last_distance) = last_distance {
                use cmp::Ordering::{Equal, Greater, Less};

                match new_distance.cmp(&last_distance) {
                    Less => {
                        if found {
                            step += 1;
                        } else {
                            step *= 2;
                        }
                    }
                    Greater => {
                        found |= true;
                        step *= -1;
                        step /= 2;
                    }
                    Equal => (),
                }
            }
            guess += step;
            last_distance = Some(new_distance);
        } else {
            unreachable!()
        }
    }

    Ok(result.to_string())
}

pub(crate) const DAY: Day = Day {
    number: 21,
    part1: solve1,
    part2: solve2,
};
