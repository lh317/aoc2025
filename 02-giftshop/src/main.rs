use std::ops::RangeInclusive;

use eyre::{OptionExt, Result, eyre};
use nom::character::complete::{char, digit1, newline};
use nom::combinator::{all_consuming, map_res};
use nom::multi::separated_list1;
use nom::sequence::terminated;
use nom::{IResult, Parser};

fn parse_range(input: &str) -> IResult<&str, RangeInclusive<isize>> {
    (
        terminated(map_res(digit1, |s: &str| s.parse::<isize>()), char('-')),
        map_res(digit1, |s: &str| s.parse::<isize>()),
    )
        .map(|(start, end)| start..=end)
        .parse(input)
}

fn parse_file(input: &str) -> IResult<&str, Vec<RangeInclusive<isize>>> {
    all_consuming(terminated(separated_list1(char(','), parse_range), newline))
        .parse(input)
}

fn is_repeated_once(num: isize) -> bool {
    let s = num.to_string();
    let half = s.len() / 2;
    s.len().is_multiple_of(2) && s[..half] == s[half..]
}

fn is_repeated_any(num: isize) -> bool {
    let s = num.to_string();
    let half = s.len() / 2;
    for base_len in 1..=half {
        if s.len().is_multiple_of(base_len) {
            let base = &s[..base_len];
            if s.matches(base).count() == s.len() / base_len {
                return true;
            }
        }
    }
    false
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body: String = std::fs::read_to_string(&fname)?;
    let ranges = match parse_file(&body) {
        Ok((_, v)) => v,
        Err(e) => match e {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                return Err(eyre!("{fname}: parsing failed: {e:?}"));
            }
        },
    };
    let first = ranges
        .iter()
        .cloned()
        .flatten()
        .filter(|num| is_repeated_once(*num))
        .sum::<isize>();
    println!("{first}");
    let second = ranges
        .iter()
        .cloned()
        .flatten()
        .filter(|num| is_repeated_any(*num))
        .sum::<isize>();
    println!("{second}");
    Ok(())
}
