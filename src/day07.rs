use std::collections::HashMap;

use itertools::Itertools;

use crate::daylib::Day;

pub(crate) mod parser {
    use color_eyre::eyre::eyre;
    use nom::sequence::preceded;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{alphanumeric1, digit1},
        combinator::{map, opt, recognize},
        multi::{many0, separated_list0},
        sequence::{delimited, separated_pair, tuple},
        Finish, IResult,
    };

    #[derive(Debug, Eq, PartialEq)]
    pub(crate) enum CdKind {
        UpOne,
        To(String),
        ToRoot,
    }

    #[derive(Debug, Eq, PartialEq)]
    pub(crate) enum DirContent {
        File { size: usize, name: String },
        Dir { name: String },
    }

    #[derive(Debug, Eq, PartialEq)]
    pub(crate) enum Command {
        Cd(CdKind),
        Ls(Vec<DirContent>),
    }

    fn parse_cd(input: &str) -> IResult<&str, Command> {
        let (i, kind) = delimited(
            tag("$ cd "),
            alt((
                map(tag("/"), |_| CdKind::ToRoot),
                map(tag(".."), |_| CdKind::UpOne),
                map(alphanumeric1, |p: &str| CdKind::To(p.to_string())),
            )),
            opt(tag("\n")),
        )(input)?;

        Ok((i, Command::Cd(kind)))
    }

    fn parse_ls(input: &str) -> IResult<&str, Command> {
        let file_line = map(
            separated_pair(
                digit1,
                tag(" "),
                recognize(tuple((
                    alphanumeric1,
                    opt(tuple((tag("."), alphanumeric1))),
                ))),
            ),
            |(size_s, name): (&str, &str)| DirContent::File {
                size: size_s.parse().unwrap(),
                name: name.to_string(),
            },
        );

        let dir_line = map(preceded(tag("dir "), alphanumeric1), |name: &str| {
            DirContent::Dir {
                name: name.to_string(),
            }
        });

        map(
            delimited(
                tag("$ ls\n"),
                separated_list0(tag("\n"), alt((dir_line, file_line))),
                opt(tag("\n")),
            ),
            Command::Ls,
        )(input)
    }

    pub(crate) fn parse(input: &str) -> color_eyre::Result<Vec<Command>> {
        many0(alt((parse_cd, parse_ls)))(input)
            .finish()
            .map(|(tail, result)| {
                assert!(
                    tail.is_empty(),
                    "unexpected non-empty tail: \n{tail}\n\nresult: {result:?}"
                );
                result
            })
            .map_err(|e| eyre!("parse error {e}"))
    }

    #[cfg(test)]
    mod tests {
        use test_case::test_case;

        use crate::day07::parser::{
            parse_cd, parse_ls,
            CdKind::{To, ToRoot, UpOne},
            Command,
            Command::{Cd, Ls},
            DirContent::{Dir, File},
        };

        #[test_case("$ cd /", Cd(ToRoot), "" ; "basic to root")]
        #[test_case("$ cd /\n", Cd(ToRoot), "" ; "basic to root with newline")]
        #[test_case("$ cd /\nfoo", Cd(ToRoot), "foo" ; "basic to root with a tail")]
        #[test_case("$ cd foo", Cd(To("foo".to_string())), "" ; "basic to path")]
        #[test_case("$ cd ..", Cd(UpOne), "" ; "basic up one")]
        fn parse_cd_tests(line: &str, result: Command, tail: &str) {
            assert_eq!(parse_cd(line), Ok((tail, result)));
        }

        #[test]
        fn parse_ls_test() {
            assert_eq!(
                parse_ls("$ ls\n12345 foo\n23 bar.txt\ndir baz"),
                Ok((
                    "",
                    Ls(vec![
                        File {
                            size: 12345,
                            name: "foo".to_string()
                        },
                        File {
                            size: 23,
                            name: "bar.txt".to_string()
                        },
                        Dir {
                            name: "baz".to_string()
                        }
                    ])
                ))
            );
        }
    }
}

#[derive(Debug)]
struct Dir {
    subdirs: HashMap<String, Box<Dir>>,
    files: HashMap<String, usize>,
}

impl Dir {
    fn new() -> Dir {
        Dir {
            subdirs: HashMap::new(),
            files: HashMap::new(),
        }
    }

    fn on(&mut self, path: &[String]) -> &mut Dir {
        let mut current = self;
        for p in path.iter() {
            current = current.subdirs.get_mut(p).unwrap();
        }
        current
    }

    fn from_commands(cs: Vec<parser::Command>) -> Dir {
        use parser::{
            CdKind::{To, ToRoot, UpOne},
            Command::{Cd, Ls},
            DirContent,
        };

        let mut root = Dir::new();
        let mut path: Vec<String> = Vec::new();

        for command in cs {
            match command {
                Cd(ToRoot) => {
                    path.drain(..);
                }
                Cd(To(dir)) => {
                    root.on(&path)
                        .subdirs
                        .entry(dir.to_string())
                        .or_insert_with(|| Box::new(Dir::new()));
                    path.push(dir.to_string());
                }
                Cd(UpOne) => {
                    drop(path.pop());
                }
                Ls(contents) => {
                    for c in &contents {
                        match c {
                            DirContent::Dir { name } => {
                                root.on(&path)
                                    .subdirs
                                    .entry(name.to_string())
                                    .or_insert_with(|| Box::new(Dir::new()));
                            }
                            DirContent::File { name, size } => {
                                root.on(&path)
                                    .files
                                    .entry(name.to_string())
                                    .or_insert(*size);
                            }
                        }
                    }
                }
            }
        }

        root
    }

    fn size(&self) -> usize {
        let subfolders = self.subdirs.values().map(|d| d.size()).sum::<usize>();
        let files = self.files.values().sum::<usize>();
        subfolders + files
    }
}

fn solve1(input: &str) -> color_eyre::Result<String> {
    fn walker(d: &Dir) -> usize {
        let raw_self_size = d.size();
        let self_size = if raw_self_size <= 100_000 {
            raw_self_size
        } else {
            0
        };

        let mut s = 0;
        for sd in d.subdirs.values() {
            s += walker(sd);
        }

        self_size + s
    }

    let commands = parser::parse(input)?;
    let root = Dir::from_commands(commands);

    Ok(walker(&root).to_string())
}

fn solve2(input: &str) -> color_eyre::Result<String> {
    let commands = parser::parse(input)?;
    let root = Dir::from_commands(commands);

    let mut dirs: Vec<&Dir> = Vec::new();
    {
        fn walker<'a>(acc: &mut Vec<&'a Dir>, d: &'a Dir) {
            acc.push(d);

            for sd in d.subdirs.values() {
                walker(acc, sd);
            }
        }
        walker(&mut dirs, &root);
    }

    let space_needed = 30_000_000 - (70_000_000 - root.size());
    Ok(dirs
        .iter()
        .map(|d| d.size())
        .sorted()
        .find(|x| *x >= space_needed)
        .unwrap()
        .to_string())
}

pub(crate) const DAY: Day = Day {
    number: 7,
    part1: solve1,
    part2: solve2,
};
