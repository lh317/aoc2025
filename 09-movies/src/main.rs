use std::cmp;

use eyre::{OptionExt, Result, eyre};
use itertools::Itertools;
use nom::character::complete::{char, digit1, newline};
use nom::combinator::{all_consuming, map, map_res, opt};
use nom::multi::separated_list1;
use nom::sequence::terminated;
use nom::{IResult, Parser};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Point2D(isize, isize);

impl Point2D {
    fn rect_area(&self, rhs: &Point2D) -> isize {
        let x = (self.0 - rhs.0).abs() + 1;
        let y = (self.1 - rhs.1).abs() + 1;
        x * y
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
struct Line {
    start: Point2D,
    end: Point2D
}

impl Line {
    fn left(&self) -> isize {
        cmp::min(self.start.0, self.end.0)
    }

    fn right(&self) -> isize {
        cmp::max(self.start.0, self.end.0)
    }

    fn top(&self) -> isize {
        cmp::min(self.start.1, self.end.1)
    }

    fn bottom(&self) -> isize {
        cmp::max(self.start.1, self.end.1)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
struct Rect {
    ul: Point2D,
    br: Point2D
}

impl Rect {
    fn left(&self) -> isize {
        self.ul.0
    }

    fn right(&self) -> isize {
        self.br.0
    }

    fn top(&self) -> isize {
        self.ul.1
    }

    fn bottom(&self) -> isize {
        self.br.1
    }

    fn intersected_by(&self, rhs: &Line) -> bool {
        self.left() < rhs.right() && self.right() > rhs.left() && self.top() < rhs.bottom() && self.bottom() > rhs.top()
    }
}

fn parse_number(input: &str) -> IResult<&str, isize> {
    map_res(digit1, |s: &str| s.parse::<isize>()).parse(input)
}

fn parse_point(input: &str) -> IResult<&str, Point2D> {
    map((
        terminated(parse_number, char(',')),
        parse_number
    ), |(x, y)| Point2D(x, y)).parse(input)
}

fn parse_file(input: &str) -> IResult<&str, Vec<Point2D>> {
    all_consuming(terminated(separated_list1(newline, parse_point), opt(newline))).parse(input)
}


fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body: String = std::fs::read_to_string(&fname)?;
    let points = match parse_file(&body) {
        Ok((_, v)) => v,
        Err(e) => match e {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                return Err(eyre!("{fname}: parsing failed: {e:?}"));
            }
        },
    };
    // Part 1
    let mut areas = points.iter().combinations(2).map(|v| (v[0].rect_area(v[1]), v[0], v[1])).collect::<Vec<_>>();
    areas.sort_unstable();
    println!("{}", areas.last().map(|(a, _, _)| a).ok_or_eyre("too few points")?);
    // Part 2
     let lines = points.iter().circular_tuple_windows().map(|(l, r)| if l <= r {
        Line{start: l.clone(), end: r.clone()}
    } else {
        Line{start: r.clone(), end: l.clone()}
    }).collect::<Vec<_>>();
    let second = areas.iter().rev().find(|(_, p1, p2)| {
        let p3 = Point2D(p1.0, p2.1);
        let p4 = Point2D(p2.0, p1.1);
        let mut points = [p1, p2, &p3, &p4];
        points.sort_unstable();
        let rect = Rect { ul: points[0].clone(), br: points[3].clone()};
        lines.iter().all(|l| !rect.intersected_by(l))
    }).map(|(a, _, _)| a).ok_or_eyre("nothing inside")?;
    println!("{second}");
    Ok(())
}
