use eyre::{Context, OptionExt, eyre};
use ndarray::{Array2, ArrayRef2};
use nom::bytes::complete::{tag, take_until, take_until1};
use nom::character::complete::{char, digit1, newline};
use nom::combinator::{all_consuming, map, map_res, opt};
use nom::multi::{many1, separated_list1};
use nom::sequence::{preceded, terminated};
use nom::{IResult, Parser};

fn parse_number<N: std::str::FromStr>(input: &str) -> IResult<&str, N> {
    map_res(digit1, |s: &str| s.parse::<N>()).parse(input)
}

fn parse_shape(input: &str) -> IResult<&str, Array2<bool>> {
    let shape = map_res(take_until1("\n\n"), |s: &str| {
        let cols = s.find("\n").unwrap();
        let rows = s.lines().count();
        let data = s
            .chars()
            .filter_map(|c| match c {
                '\n' => None,
                '#' => Some(Ok(true)),
                '.' => Some(Ok(false)),
                _ => Some(Err(eyre!("unknown shape char: {c}"))),
            })
            .collect::<eyre::Result<Vec<_>>>()?;
        Array2::from_shape_vec((rows, cols), data).context("failed to create array")
    });
    terminated(shape, tag("\n\n")).parse(input)
}

fn parse_present(input: &str) -> IResult<&str, Array2<bool>> {
    preceded((take_until(":\n"), tag(":\n")), parse_shape).parse(input)
}

#[derive(Debug, Clone)]
struct Region {
    width: usize,
    height: usize,
    presents: Vec<usize>,
}

fn parse_region(input: &str) -> IResult<&str, Region> {
    let parser = (
        terminated(parse_number, char('x')),
        terminated(parse_number, tag(": ")),
        separated_list1(char(' '), parse_number),
    );
    map(parser, |(width, height, presents)| Region { width, height, presents })
        .parse(input)
}

fn parse_file(input: &str) -> IResult<&str, (Vec<Array2<bool>>, Vec<Region>)> {
    let parser = terminated(
        (many1(parse_present), separated_list1(newline, parse_region)),
        opt(newline),
    );
    all_consuming(parser).parse(input)
}

fn area(shape: &ArrayRef2<bool>) -> usize {
    shape.iter().filter(|&&e| e).count()
}

fn main() -> eyre::Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body: String = std::fs::read_to_string(&fname)?;
    let (presents, regions) = match parse_file(&body) {
        Ok((_, v)) => v,
        Err(e) => match e {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                return Err(eyre!("{fname}: parsing failed: {e:?}"));
            }
        },
    };
    for region in regions.iter() {
        if region.presents.len() != presents.len() {
            return Err(eyre!("region contains wrong number of presents"))
        }
    }
    let areas = presents.iter().map(|p| area(p)).collect::<Vec<_>>();
    let usable = regions.iter().filter(|r| {
        let area = r.width * r.height;
        let needed = r.presents.iter().zip(areas.iter()).map(|(&p, &a)| p * a).sum();
        area >= needed
    }).count();
    println!("{usable}");
    Ok(())
}
