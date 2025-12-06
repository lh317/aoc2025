use std::str::FromStr;

use eyre::{OptionExt, Result, eyre};
use ndarray::{Array, Array2, Axis};
use nom::bytes::complete::take_while;
use nom::character::complete::one_of;
use nom::combinator::{map_res, recognize};
use nom::multi::many1;
use nom::{IResult, Parser};

#[derive(Debug)]
enum OpKind {
    Add,
    Multiply,
}

#[derive(Debug)]
struct Op {
    len: u8,
    kind: OpKind,
}

impl FromStr for Op {
    type Err = eyre::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut chars = s.chars();
        let kind = match chars.nth(0) {
            Some('*') => Ok(OpKind::Multiply),
            Some('+') => Ok(OpKind::Add),
            Some(_) => Err(eyre!("unknown operator {s:?}")),
            None => Err(eyre!("column too short: {s:?}")),
        }?;
        // s contains the space between columns but one character was consumed above.
        let len = u8::try_from(chars.count())?;
        if len > 0 {
            Ok(Self { len, kind })
        } else {
            Err(eyre!("column too short: {s:?}"))
        }
    }
}

fn parse_op_line(input: &str) -> IResult<&str, Vec<Op>> {
    // newline counts as trailing space for last entry, needed for from_str above.
    let parse_op = map_res(
        recognize((one_of("+*"), take_while(|c| " \n".contains(c)))),
        |s: &str| s.parse::<Op>(),
    );
    many1(parse_op).parse(input)
}

fn parse_num_lines<'a>(cols: &[Op], input: &'a str) -> Result<Vec<&'a str>> {
    let mut result = Vec::with_capacity(cols.len() * 4); // cheat
    for (lineno, line) in input.lines().enumerate() {
        let mut start = 0usize;
        for col in cols {
            let end = start + usize::from(col.len);
            if end > line.len() {
                return Err(eyre!("{}: line too short", lineno + 1));
            }
            result.push(&line[start..end]);
            start = end + 1; // skip space between columns
        }
        if start < line.len() {
            return Err(eyre!(
                "{}: did not parse enough {}",
                lineno + 1,
                line.len() - start
            ));
        }
    }
    Ok(result)
}

fn part1(ops: &[Op], num_strs: &[&str]) -> Result<usize> {
    let columns = ops.len();
    let rows = num_strs.len() / columns;
    let num_vec = num_strs
        .iter()
        .map(|s| Ok(s.trim().parse::<usize>()?))
        .collect::<Result<Vec<_>>>()?;
    let nums = Array2::from_shape_vec((rows, columns), num_vec)?;
    let sum = ops
        .iter()
        .zip(nums.axis_iter(Axis(1)))
        .map(|(op, axis)| match op.kind {
            OpKind::Add => axis.sum(),
            OpKind::Multiply => axis.product(),
        })
        .sum::<usize>();
    Ok(sum)
}

fn part2(ops: &[Op], num_strs: &[&str]) -> Result<usize> {
    let columns = ops.len();
    let rows = num_strs.len() / columns;
    let fields =
        Array::from_iter(num_strs.iter()).into_shape_with_order((rows, columns))?;
    let sum = ops
        .iter()
        .zip(fields.axis_iter(Axis(1)))
        .map(|(op, axis)| {
            let mut nums = vec![0usize; op.len.into()];
            for (i, num) in nums.iter_mut().enumerate() {
                for field in axis.iter() {
                    if let Some(digit) = field.chars().nth(i).unwrap().to_digit(10) {
                        *num = 10 * *num + usize::try_from(digit).unwrap();
                    }
                }
            }
            match op.kind {
                OpKind::Add => nums.iter().sum::<usize>(),
                OpKind::Multiply => nums.iter().product(),
            }
        })
        .sum::<usize>();
    Ok(sum)
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body: String = std::fs::read_to_string(&fname)?;
    let ops_start =
        body.find(['*', '+']).ok_or_eyre("{fname}: could not find op line")?;
    let ops = match parse_op_line(&body[ops_start..]) {
        Ok((_, v)) => v,
        Err(e) => match e {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                return Err(eyre!("{fname}: parsing failed: {e:?}"));
            }
        },
    };
    let raw_nums = parse_num_lines(&ops, &body[..ops_start])?;
    println!("{}", part1(&ops, &raw_nums)?);
    println!("{}", part2(&ops, &raw_nums)?);
    Ok(())
}
