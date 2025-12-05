use std::cmp::{max, min};

use eyre::{Context, OptionExt, Result, eyre};
use ndarray::{s, Array2, ArrayView2};

fn parse_file(body: &str, file_name: Option<&str>) -> Result<Array2<bool>> {
    let mut rows = 0usize;
    let mut columns = 0usize;
    let mut values = Vec::new();
    for line in body.lines() {
        rows += 1;
        for c in line.chars() {
            values.push(c == '@');
        }
        if rows == 1 {
            columns = line.len();
        } else if line.len() != columns {
            return Err(eyre!("{}:{rows}: line not of expected length {columns}", file_name.unwrap_or("unknown")));
        }
    }
    Array2::from_shape_vec((rows, columns), values).wrap_err("error creating array")
}

fn count_accessible(rolls: ArrayView2<bool>) -> usize {
    let mut count = 0usize;
    for ((y, x), is_roll) in rolls.indexed_iter() {
        if !*is_roll {
            continue;
        }
        let ul = if y >= 1 && x >= 1 {
            rolls[[y-1, x-1]] as u8
        } else {
            0u8
        };
        let u = if y >= 1 {
            rolls[[y-1, x]] as u8
        } else {
            0u8
        };
        let ur = if y >= 1 && x + 1 < rolls.ncols()  {
            rolls[[y-1, x+1]] as u8
        } else {
            0u8
        };
        let l = if x >= 1 {
            rolls[[y, x-1]] as u8
        } else {
            0u8
        };
        let r = if x + 1 < rolls.ncols() {
            rolls[[y, x+1]] as u8
        } else {
            0u8
        };
        let bl = if y + 1 < rolls.nrows() && x >= 1 {
            rolls[[y+1, x-1]] as u8
        } else {
            0u8
        };
        let b = if y + 1 < rolls.nrows() {
            rolls[[y+1, x]] as u8
        } else {
            0u8
        };
        let br = if y + 1 < rolls.nrows() && x + 1 < rolls.ncols() {
            rolls[[y+1, x+1]] as u8
        } else {
            0u8
        };
        if ul + u + ur + l + r + bl + b + br < 4 {
            count += 1;
        }
    }
    count
}


fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body: String = std::fs::read_to_string(&fname)?;
    let rolls = parse_file(&body, Some(&fname))?;
    println!("{}", count_accessible(rolls.view()));
    Ok(())
}
