use std::cmp::max;
use std::ops::RangeInclusive;

use eyre::{OptionExt, Result, eyre};
use nom::character::complete::{char, digit1, newline};
use nom::combinator::{all_consuming, map_res, opt};
use nom::multi::separated_list1;
use nom::sequence::terminated;
use nom::{IResult, Parser};

fn parse_number(input: &str) -> IResult<&str, isize> {
    map_res(digit1, |s: &str| s.parse::<isize>()).parse(input)
}

fn parse_range(input: &str) -> IResult<&str, RangeInclusive<isize>> {
    (terminated(parse_number, char('-')), parse_number)
        .map(|(start, end)| start..=end)
        .parse(input)
}

fn parse_file(input: &str) -> IResult<&str, (Vec<RangeInclusive<isize>>, Vec<isize>)> {
    let ranges = separated_list1(newline, parse_range);
    let ingredients = separated_list1(newline, parse_number);
    all_consuming((
        terminated(ranges, (newline, newline)),
        terminated(ingredients, opt(newline)),
    ))
    .parse(input)
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body: String = std::fs::read_to_string(&fname)?;
    let (mut ranges, ingredients) = match parse_file(&body) {
        Ok((_, v)) => v,
        Err(e) => match e {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                return Err(eyre!("{fname}: parsing failed: {e:?}"));
            }
        },
    };
    ranges.sort_by_key(|r| (*r.start(), *r.end()));
    let fresh = ingredients
        .iter()
        .filter_map(|id| ranges.iter().find(|r| r.contains(id)))
        .count();
    println!("{fresh}");
    if ranges.is_empty() {
        return Err(eyre!("{fname}: no input ranges"));
    }
    let mut total = 0isize;
    let mut current = ranges[0].clone();
    for range in ranges {
        if current.contains(range.start()) {
            current = *current.start()..=max(*current.end(), *range.end())
        } else {
            total += current.end() - current.start() + 1;
            current = range;
        }
    }
    total += current.end() - current.start() + 1;
    println!("{total}");
    Ok(())
}
