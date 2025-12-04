#![allow(dead_code)]
#![allow(clippy::needless_range_loop)]
use eyre::{OptionExt, Result, eyre};
use nom::character::complete::one_of;
use nom::multi::many1;
use nom::{IResult, Parser};
use nom::combinator::{all_consuming, map_opt};

fn parse_line(input: &str) -> IResult<&str, Vec<u8>> {
    all_consuming(many1(map_opt(one_of("0123456789"), |c| c.to_digit(10).map(|d| d as u8)))).parse(input)
}


fn max_joltage(digits: impl IntoIterator<Item = u8, IntoIter: DoubleEndedIterator + Clone>) -> Option<u8> {
    let iter = digits.into_iter();
    let first = iter.clone().rev().enumerate().skip(1).max_by_key(|(_, d)| *d);
    if let Some((i, tens)) = first {
        let second = iter.rev().take(i).max();
        second.map(|ones| tens * 10 + ones)
    } else {
        None
    }
}

fn dangerous_joltage<const N: usize>(digits: impl IntoIterator<Item = u8, IntoIter: ExactSizeIterator + DoubleEndedIterator + Clone>) -> Option<isize> {
    let iter = digits.into_iter();
    if iter.len() >= N {
        let mut places = [0u8; N];
        let mut pos = iter.len();
        for i in 0..N {
            (pos, places[i]) = iter.clone().rev().enumerate().take(pos).skip(N - 1 - i).max_by_key(|(_, d)| *d)?;
        }
        let mut result = 0isize;
        for d in places {
            result = result * 10 + isize::from(d);
        }
        Some(result)
    } else {
        None
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body: String = std::fs::read_to_string(&fname)?;
    let mut sum_joltage = 0isize;
    let mut sum_dangerous = 0isize;
    for (lineno, line) in body.lines().enumerate() {
        let lineno = lineno + 1;
        let bank = match parse_line(line) {
            Ok((_, v)) => v,
            Err(e) => match e {
                nom::Err::Incomplete(_) => unreachable!(),
                nom::Err::Error(e) | nom::Err::Failure(e) => return Err(eyre!("{fname}:{lineno}: parsing failed: {e:?}")),
            }
        };
        //let joltage = max_joltage(bank.iter().cloned()).ok_or_eyre("{fname}:{lineno}: too short")?;
        let joltage = dangerous_joltage::<2>(bank.iter().cloned()).ok_or_eyre("{fname}:{lineno}: too short")?;
        let dangerous = dangerous_joltage::<12>(bank).ok_or_eyre("{fname}:{lineno}: too short")?;
        sum_joltage += joltage;
        sum_dangerous += dangerous;
    }
    println!("{sum_joltage}");
    println!("{sum_dangerous}");
    Ok(())
}
