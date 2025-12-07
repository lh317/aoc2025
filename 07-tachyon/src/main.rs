use std::collections::HashSet;
use eyre::{OptionExt, Result, eyre};
use ndarray::{Array2, ArrayRef2, Axis};

#[derive(Debug, PartialEq)]
enum Element {
    Empty,
    Start,
    Splitter,
    //Beam
}

impl TryFrom<char> for Element {
    type Error = eyre::Error;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        match value {
            '.' => Ok(Self::Empty),
            'S' => Ok(Self::Start),
            '^' => Ok(Self::Splitter),
            _ => Err(eyre!("unknown element: {value}"))
        }
    }
}

fn parse_file(input: &str) -> Result<(Array2<Element>, (usize, usize))> {
    let mut start = None;
    let mut rows = 0usize;
    let mut columns = 0usize;
    let mut data = Vec::<Element>::new();
    for line in input.lines() {
        if rows == 0 {
            columns = line.len()
        } else if line.len() != columns {
            return Err(eyre!("{}: line not expected length {columns}", rows + 1));
        }
        for (i, c) in line.chars().enumerate() {
            let element = c.try_into()?;
            if element == Element::Start {
                if let Some((y, x)) = start {
                    return Err(eyre!("{}:{}: duplicate start found, original at coords {x}, {y}", rows + 1, i + 1));
                } else {
                    start = Some((rows, i));
                }
            }
            data.push(element);
        }
        rows += 1;
    }
    let array = Array2::from_shape_vec((rows, columns), data)?;
    start.map(|pos| (array, pos)).ok_or_eyre("start never found")
}

fn split_beam((start_y, start_x): (usize, usize), array: &ArrayRef2<Element>) -> Option<usize> {
    let mut splits = 0usize;
    if start_y + 1 >= array.nrows() || start_x >= array.ncols(){
        return None;
    }
    let mut beams = HashSet::new();
    beams.insert(start_x);
    for row in array.axis_iter(Axis(0)).skip(start_y + 1) {
        let splitters = row.indexed_iter().filter_map(|(x, elem)| if *elem == Element::Splitter {
            Some(x)
        } else {
            None
        });
        for splitter in splitters {
            if beams.contains(&splitter) {
                beams.remove(&splitter);
                if splitter > 0 {
                   beams.insert(splitter - 1);
                }
                if splitter + 1 < array.ncols() {
                    beams.insert(splitter + 1);
                }
                splits += 1;
            }
        }
    }
    Some(splits)
}

fn timelines((start_y, start_x): (usize, usize), array: &ArrayRef2<Element>) -> Option<usize> {
    if start_y + 1 >= array.nrows() || start_x >= array.ncols() {
        return None;
    }
    let mut timelines = vec![0usize; array.ncols()];
    timelines[start_x] = 1;
    for row in array.axis_iter(Axis(0)).skip(start_y + 1) {
        let splitters = row.indexed_iter().filter_map(|(x, elem)| if *elem == Element::Splitter {
            Some(x)
        } else {
            None
        });
        for splitter in splitters {
            if timelines[splitter] > 0 {
                if splitter > 0 {
                   timelines[splitter - 1] += timelines[splitter];
                }
                if splitter + 1 < array.ncols() {
                    timelines[splitter + 1] += timelines[splitter];
                }
                timelines[splitter] = 0;
            }
        }
    }
    Some(timelines.into_iter().sum())
}


fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body: String = std::fs::read_to_string(&fname)?;
    let (array, start) = parse_file(&body)?;
    println!("{}", split_beam(start, &array).unwrap());
    println!("{}", timelines(start, &array).unwrap());
    Ok(())
}
