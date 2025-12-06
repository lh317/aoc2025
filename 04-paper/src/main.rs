use eyre::{Context, OptionExt, Result, eyre};
use ndarray::{Array2, ArrayRef2};

fn in_bounds((y, x): (isize, isize), (rows, cols): (isize, isize)) -> bool {
    y >= 0 && y < rows && x >= 0 && x < cols
}

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
            return Err(eyre!(
                "{}:{rows}: line not of expected length {columns}",
                file_name.unwrap_or("unknown")
            ));
        }
    }
    Array2::from_shape_vec((rows, columns), values).wrap_err("error creating array")
}

fn is_accessible((y, x): (usize, usize), rolls: &ArrayRef2<bool>) -> bool {
    let y = isize::try_from(y).expect("array too large");
    let x = isize::try_from(x).expect("array too large");
    let rows = isize::try_from(rolls.nrows()).expect("array too large");
    let cols = isize::try_from(rolls.ncols()).expect("array too large");
    let pos = [
        (y - 1, x - 1),
        (y - 1, x),
        (y - 1, x + 1),
        (y, x - 1),
        (y, x + 1),
        (y + 1, x - 1),
        (y + 1, x),
        (y + 1, x + 1),
    ];
    pos.iter()
        .filter(|&&(y, x)| in_bounds((y, x), (rows, cols)) && rolls[[y as usize, x as usize]])
        .count()
        < 4
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body: String = std::fs::read_to_string(&fname)?;
    let mut rolls = parse_file(&body, Some(&fname))?;
    let mut removed = rolls
        .indexed_iter()
        .filter_map(|(pos, c)| {
            if *c && is_accessible(pos, &rolls) {
                Some(pos)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let mut count = removed.len();
    println!("{}", count);
    loop {
        if removed.is_empty() {
            println!("{count}");
            return Ok(());
        }
        for pos in removed {
            rolls[pos] = false;
        }
        removed = rolls
            .indexed_iter()
            .filter_map(|(pos, c)| {
                if *c && is_accessible(pos, &rolls) {
                    Some(pos)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        count += removed.len();
    }
}
