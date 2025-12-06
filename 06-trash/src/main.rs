use std::str::FromStr;

use eyre::{Context, OptionExt, Result, eyre};
use ndarray::{Array2, Axis};
use nom::bytes::complete::take_while;
use nom::character::complete::{digit1, newline, one_of};
use nom::combinator::{all_consuming, map_res, opt, recognize};
use nom::multi::{fold_many1, separated_list1};
use nom::sequence::terminated;
use nom::{IResult, Parser};

enum Op {
    Add,
    Multiply,
}

impl FromStr for Op {
    type Err = eyre::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "*" => Ok(Self::Multiply),
            "+" => Ok(Self::Add),
            _ => Err(eyre!("unknown operator {s:?}")),
        }
    }
}

fn parse_number(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse::<usize>()).parse(input)
}

fn parse_num_line(input: &str) -> IResult<&str, Vec<usize>> {
    terminated(separated_list1(take_while(|c| c == ' '), parse_number), take_while(|c| c == ' '))
        .parse(input)
}

fn parse_op_line(input: &str) -> IResult<&str, Vec<Op>> {
    let parse_op = map_res(recognize(one_of("*+")), |s: &str| s.parse::<Op>());
    terminated(separated_list1(take_while(|c| c == ' '), parse_op), take_while(|c| c == ' ')).parse(input)
}

fn parse_file(input: &str) -> IResult<&str, (Vec<usize>, Vec<Op>)> {
    all_consuming((
        fold_many1(terminated(parse_num_line, newline), Vec::new, |mut acc, v| {
            acc.extend(v);
            acc
        }),
        terminated(parse_op_line, opt(newline)),
    ))
    .parse(input)
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body: String = std::fs::read_to_string(&fname)?;
    let (raw_nums, ops) = match parse_file(&body) {
        Ok((_, v)) => v,
        Err(e) => match e {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                return Err(eyre!("{fname}: parsing failed: {e:?}"));
            }
        },
    };
    let columns = ops.len();
    let rows = raw_nums.len() / columns;
    let nums =
        Array2::from_shape_vec((rows, columns), raw_nums).wrap_err("error creating array")?;
    let total = ops
        .iter()
        .zip(nums.axis_iter(Axis(1)))
        .map(|(op, axis)| match op {
            Op::Add => axis.sum(),
            Op::Multiply => axis.product(),
        })
        .sum::<usize>();
    println!("{total}");
    Ok(())
}
