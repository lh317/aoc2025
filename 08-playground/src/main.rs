use std::collections::HashMap;

use eyre::{OptionExt, Result, eyre};
use itertools::Itertools;
use nom::character::complete::{char, digit1, newline};
use nom::combinator::{all_consuming, map, map_res, opt};
use nom::multi::separated_list1;
use nom::sequence::terminated;
use nom::{IResult, Parser};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Point3D(isize, isize, isize);

fn parse_number(input: &str) -> IResult<&str, isize> {
    map_res(digit1, |s: &str| s.parse::<isize>()).parse(input)
}

impl Point3D {
    fn distance(&self, rhs: &Point3D) -> f64 {
        let x = self.0 - rhs.0;
        let y = self.1 - rhs.1;
        let z = self.2 - rhs.2;
        ((x * x + y * y + z * z) as f64).sqrt()
    }
}


fn parse_point(input: &str) -> IResult<&str, Point3D> {
    map((
        terminated(parse_number, char(',')),
        terminated(parse_number, char(',')),
        parse_number
    ), |(x, y, z)| Point3D(x, y, z)).parse(input)
}

fn parse_file(input: &str) -> IResult<&str, Vec<Point3D>> {
    all_consuming(terminated(separated_list1(newline, parse_point), opt(newline))).parse(input)
}

fn to_circuits(pairs: impl IntoIterator<Item = (Point3D, Point3D)>, n: usize) -> HashMap<usize, Vec<Point3D>> {
    let mut id = 0usize;
    let mut circuits = HashMap::new();
    let mut points = HashMap::new();
    for (lhs, rhs) in pairs.into_iter().take(n) {
        match (points.get(&lhs), points.get(&rhs)) {
            (None, None) => {
                circuits.insert(id, vec![lhs.clone(), rhs.clone()]);
                points.insert(lhs.clone(), id);
                points.insert(rhs.clone(), id);
                id += 1;
            },
            (Some(id), None) => {
                circuits.get_mut(id).unwrap().push(rhs.clone());
                points.insert(rhs.clone(), *id);
            },
            (None, Some(id)) => {
                circuits.get_mut(id).unwrap().push(lhs.clone());
                points.insert(lhs.clone(), *id);
            },
            (Some(&lhs_id), Some(&rhs_id)) => {
                if lhs_id != rhs_id {
                    let mut removed = circuits.remove(&rhs_id).unwrap();
                    for point in removed.iter() {
                        *points.get_mut(point).unwrap() = lhs_id;
                    }
                    circuits.get_mut(&lhs_id).unwrap().append(&mut removed);
                }
            }
        }
    }
    circuits
}

fn until_connected(pairs: impl IntoIterator<Item =(Point3D, Point3D)>, num_points: usize) -> Option<(Point3D, Point3D)>
{
    let mut id = 0usize;
    let mut circuits = HashMap::new();
    let mut points = HashMap::new();
    for (lhs, rhs) in pairs.into_iter() {
        match (points.get(&lhs), points.get(&rhs)) {
            (None, None) => {
                circuits.insert(id, vec![lhs.clone(), rhs.clone()]);
                points.insert(lhs.clone(), id);
                points.insert(rhs.clone(), id);
                id += 1;
            },
            (Some(id), None) => {
                let dest = circuits.get_mut(id).unwrap();
                dest.push(rhs.clone());
                if dest.len() == num_points {
                    return Some((lhs, rhs));
                }
                points.insert(rhs.clone(), *id);
            },
            (None, Some(id)) => {
                let dest = circuits.get_mut(id).unwrap();
                dest.push(lhs.clone());
                if dest.len() == num_points {
                    return Some((lhs, rhs));
                }
                points.insert(lhs.clone(), *id);
            },
            (Some(&lhs_id), Some(&rhs_id)) => {
                if lhs_id != rhs_id {
                    let mut removed = circuits.remove(&rhs_id).unwrap();
                    for point in removed.iter() {
                        *points.get_mut(point).unwrap() = lhs_id;
                    }
                    let dest = circuits.get_mut(&lhs_id).unwrap();
                    dest.append(&mut removed);
                    if dest.len() == num_points {
                        return Some((lhs, rhs));
                    }
                    circuits.get_mut(&lhs_id).unwrap().append(&mut removed);
                }
            }
        }
    }
    None
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
    let num_points = points.len();
    let mut pairs = points.into_iter().combinations(2).map(|v| (v[0].clone(), v[1].clone())).collect::<Vec<_>>();
    pairs.sort_unstable_by(|(lhs0, lhs1), (rhs0, rhs1)| {
        let d0 = lhs0.distance(lhs1);
        let d1 = rhs0.distance(rhs1);
        d0.total_cmp(&d1)
    });
    let circuits = to_circuits(pairs.clone(), 1000);
    let lengths = circuits.values().map(|v| v.len()).sorted_unstable();
    println!("{}", lengths.rev().take(3).product::<usize>());
    let connected = until_connected(pairs, num_points);
    if let Some((lhs, rhs)) = connected {
        println!("{}", lhs.0 * rhs.0);
    } else {
        println!("search failed");
    }
    Ok(())
}
