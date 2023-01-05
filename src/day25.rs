use crate::daylib::Day;

fn snafu_dig_to_dec(c: char) -> isize {
    match c {
        '2' => 2,
        '1' => 1,
        '0' => 0,
        '-' => -1,
        '=' => -2,
        _ => panic!("unknown snafu digit {c}"),
    }
}

fn dec_dig_to_snafu(x: isize) -> char {
    match x {
        2 => '2',
        1 => '1',
        0 => '0',
        -1 => '-',
        -2 => '=',
        _ => panic!("can't translate {x} to snafu"),
    }
}

fn snafu_to_dec(x: &str) -> isize {
    x.chars()
        .map(snafu_dig_to_dec)
        .rev()
        .enumerate()
        .fold(0, |acc, (i, x)| {
            acc + x * 5isize.pow(u32::try_from(i).unwrap())
        })
}

fn dec_to_snafu(mut x: isize) -> String {
    let mut s = String::new();
    while x > 0 {
        let snafu_digit_dec = (x + 2) % 5 - 2;
        let carryover = if snafu_digit_dec < 0 {
            -snafu_digit_dec
        } else {
            0
        };

        s.push(dec_dig_to_snafu(snafu_digit_dec));
        x = (x + carryover) / 5;
    }
    s.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::day25::{dec_to_snafu, snafu_to_dec};

    #[test_case("1" => 1)]
    #[test_case("1=" => 3)]
    #[test_case("1-" => 4)]
    #[test_case("12" => 7)]
    #[test_case("20" => 10)]
    #[test_case("1=0" => 15)]
    #[test_case("1-0" => 20)]
    #[test_case("1=11-2" => 2022)]
    #[test_case("1-0---0" => 12345)]
    #[test_case("1121-1110-1=0" => 314_159_265)]
    fn snafu_to_dec_tests(x: &str) -> isize {
        snafu_to_dec(x)
    }

    #[test_case(1 => "1")]
    #[test_case(3 => "1=")]
    #[test_case(4 => "1-")]
    #[test_case(7 => "12")]
    #[test_case(10 => "20")]
    #[test_case(15 => "1=0")]
    #[test_case(20 => "1-0")]
    #[test_case(2022 => "1=11-2")]
    #[test_case(12345 => "1-0---0")]
    #[test_case(314_159_265 => "1121-1110-1=0")]
    fn dec_to_snafu_tests(x: isize) -> String {
        dec_to_snafu(x)
    }
}

#[allow(clippy::unnecessary_wraps)]
fn solve1(input: &str) -> color_eyre::Result<String> {
    Ok(dec_to_snafu(input.lines().map(snafu_to_dec).sum()))
}

#[allow(clippy::unnecessary_wraps)]
fn solve2(_input: &str) -> color_eyre::Result<String> {
    Ok("Celebration!".to_string())
}

pub(crate) const DAY: Day = Day {
    number: 25,
    part1: solve1,
    part2: solve2,
};
