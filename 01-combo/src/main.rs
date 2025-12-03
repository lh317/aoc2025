use eyre::{OptionExt, Result, eyre};
use nom::{IResult, Parser};
use nom::character::complete::{digit1, one_of};
use nom::combinator::{all_consuming, map_res};

fn parse_line<'a>(input: &'a str) -> IResult<&'a str, (char, isize)>
{

        all_consuming((
                one_of("LR"),
                map_res(digit1, |s: &'a str| s.parse::<isize>())
            )
        ).parse(input)
}


fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body: String = std::fs::read_to_string(fname.as_str())?;
    let mut zeros1 = 0isize;
    let mut zeros2 = 0isize;
    let mut dial = 50isize;
    for (lineno, line) in body.lines().enumerate() {
        let lineno = lineno + 1;
        let result = parse_line(line);
        let (dir, clicks) = match result {
            Ok((_, v)) => v,
            Err(e) => match e {
                nom::Err::Incomplete(_) => unreachable!(),
                nom::Err::Error(e) | nom::Err::Failure(e) => return Err(eyre!("{fname}:{lineno}: parsing failed: {e:?}")),
            }
        };
        let clicks = match dir {
            'L' => -clicks,
            'R' => clicks,
            _ => unreachable!()
        };
        let spins = (dial + clicks).div_euclid(100);
        let new_dial = (dial + clicks).rem_euclid(100);
        if new_dial == 0 {
            zeros1 += 1;
        }
        zeros2 += spins.abs();
        // Account for when spins is too high due to rounding towards inf:
        // If going left, when starting from 0
        // If going right, when landing on 0.
        if (spins < 0 && dial == 0) || (spins > 0 && new_dial == 0) {
            zeros2 -= 1;
        }
        dial = new_dial;
    }
    zeros2 += zeros1;  // Landing on zero should also be included.
    println!("{zeros1}");
    println!("{zeros2}");
    Ok(())
}
