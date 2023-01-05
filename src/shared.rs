use std::str::FromStr;

use nom::{
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{map_res, opt, recognize},
    sequence::preceded,
    IResult,
};

pub(crate) fn parse_unum<T: FromStr>(i: &str) -> IResult<&str, T> {
    let (i, number) = map_res(digit1, str::parse)(i)?;

    Ok((i, number))
}

pub(crate) fn parse_inum<T: FromStr>(i: &str) -> IResult<&str, T> {
    let (i, number) = map_res(recognize(preceded(opt(tag("-")), digit1)), |s: &str| {
        s.parse()
    })(i)?;

    Ok((i, number))
}
