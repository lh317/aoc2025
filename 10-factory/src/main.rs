use std::collections::HashMap;

use eyre::{OptionExt, eyre};
use itertools::Itertools;
use nom::bytes::complete::is_a;
use nom::character::complete::{char, digit1, newline, space1};
use nom::combinator::{all_consuming, map, map_res, opt};
use nom::multi::separated_list1;
use nom::sequence::{delimited, terminated};
use nom::{IResult, Parser};

#[derive(Debug)]
struct Machine {
    indicators: Vec<bool>,
    buttons: Vec<Vec<u16>>,
    joltages: Vec<u16>,
}

impl Machine {
    fn new(
        indicators: Vec<bool>,
        buttons: Vec<Vec<u16>>,
        joltages: Vec<u16>,
    ) -> eyre::Result<Self> {
        if buttons.iter().flatten().all(|i| usize::from(*i) < indicators.len()) {
            Ok(Self { indicators, buttons, joltages })
        } else {
            Err(eyre!("button has too large indicator index"))
        }
    }

    fn num_buttons(&self) -> usize {
        self.buttons.len()
    }
}

fn parse_number<N: std::str::FromStr>(input: &str) -> IResult<&str, N> {
    map_res(digit1, |s: &str| s.parse::<N>()).parse(input)
}

fn parse_indicators(input: &str) -> IResult<&str, Vec<bool>> {
    map(delimited(char('['), is_a("#."), char(']')), |s: &str| {
        s.chars()
            .map(|c| match c {
                '#' => true,
                '.' => false,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>()
    })
    .parse(input)
}

fn parse_button(input: &str) -> IResult<&str, Vec<u16>> {
    delimited(char('('), separated_list1(char(','), parse_number), char(')'))
        .parse(input)
}

fn parse_joltages(input: &str) -> IResult<&str, Vec<u16>> {
    delimited(char('{'), separated_list1(char(','), parse_number), char('}'))
        .parse(input)
}

fn parse_machine(input: &str) -> IResult<&str, Machine> {
    let parser = (
        terminated(parse_indicators, space1),
        terminated(separated_list1(space1, parse_button), space1),
        parse_joltages,
    );
    map_res(parser, |(i, b, j)| Machine::new(i, b, j)).parse(input)
}

fn parse_file(input: &str) -> IResult<&str, Vec<Machine>> {
    all_consuming(terminated(separated_list1(newline, parse_machine), opt(newline)))
        .parse(input)
}

struct ButtonStarter<'a> {
    goal: &'a [bool],
    buttons: &'a [Vec<u16>],
    indicators: Vec<bool>,
    counts: Vec<u16>,
}

impl<'a> ButtonStarter<'a> {
    fn indicators_ok(&self) -> bool {
        self.indicators == self.goal
    }

    fn num_buttons(&self) -> usize {
        self.buttons.len()
    }

    fn press_button(&mut self, index: usize) {
        let button = &self.buttons[index];
        for indicator in button {
            let indicator = usize::from(*indicator);
            self.indicators[indicator] = !self.indicators[indicator];
            self.counts[indicator] += 1;
        }
    }

    fn into_state(self) -> (Vec<bool>, Vec<u16>) {
        (self.indicators, self.counts)
    }
}

impl<'a> std::fmt::Debug for ButtonStarter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ButtonStarter")
            .field("goal", &self.goal)
            .field("indicators", &self.indicators)
            .field("counts", &self.counts)
            .finish()
    }
}

impl<'a> From<&'a Machine> for ButtonStarter<'a> {
    fn from(machine: &'a Machine) -> Self {
        let goal = &machine.indicators;
        let goal_len = goal.len();
        Self {
            goal,
            buttons: &machine.buttons,
            indicators: vec![false; goal_len],
            counts: vec![0; goal_len],
        }
    }
}

#[derive(Debug, Clone)]
struct JoltageStarter {
    joltages: Vec<u16>,
}

impl JoltageStarter {
    fn as_indicators(&self) -> impl Iterator<Item = bool> {
        self.joltages.iter().map(|j| *j % 2 == 1)
    }

    fn halve_joltages(&mut self) {
        for j in &mut self.joltages {
            *j /= 2;
        }
    }

    fn count_down(&mut self, counts: &[u16]) -> Option<bool> {
        for (j, c) in self.joltages.iter_mut().zip(counts.iter()) {
            *j = j.checked_sub(*c)?;
        }
        Some(self.joltages.iter().all(|j| *j == 0))
    }
}

impl From<&Machine> for JoltageStarter {
    fn from(machine: &Machine) -> Self {
        let joltages = machine.joltages.clone();
        Self { joltages }
    }
}

fn shortest_buttons(machine: &Machine) -> Option<usize> {
    (1..=machine.num_buttons())
        .flat_map(|n| (0..machine.num_buttons()).combinations(n))
        .filter_map(|indices| {
            let mut starter = ButtonStarter::from(machine);
            for index in &indices {
                starter.press_button(*index);
            }
            if starter.indicators_ok() { Some(indices.len()) } else { None }
        })
        .next()
}

type IndicatorsMap = HashMap<Vec<bool>, Vec<(usize, Vec<u16>)>>;
fn all_indicators(machine: &Machine) -> IndicatorsMap {
    let end = machine.num_buttons();
    let mut map = (1..=end).flat_map(|n| (0..end).combinations(n)).fold(
        HashMap::new(),
        |mut map: IndicatorsMap, indices| {
            let mut starter = ButtonStarter::from(machine);
            for index in &indices {
                starter.press_button(*index % starter.num_buttons());
            }
            let (key, counts) = starter.into_state();
            map.entry(key).or_default().push((indices.len(), counts));
            map
        },
    );
    map.entry(vec![false; machine.indicators.len()])
        .or_default()
        .push((0, vec![0; machine.indicators.len()]));
    map
}

fn shortest_joltages(machine: &Machine) -> Option<usize> {
    let indicators = all_indicators(machine);

    fn inner(starter: &JoltageStarter, lookup: &IndicatorsMap) -> Option<usize> {
        let indicators = starter.as_indicators().collect::<Vec<_>>();
        lookup
            .get(&indicators)
            .iter()
            .copied()
            .flatten()
            .filter_map(|(clicks, counts)| {
                let mut starter = starter.clone();
                if starter.count_down(counts)? {
                    Some(*clicks)
                } else {
                    starter.halve_joltages();
                    inner(&starter, lookup).map(|c| 2 * c + clicks)
                }
            })
            .min()
    }

    let starter = machine.into();
    inner(&starter, &indicators)
}

fn main() -> eyre::Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body: String = std::fs::read_to_string(&fname)?;
    let machines = match parse_file(&body) {
        Ok((_, v)) => v,
        Err(e) => match e {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                return Err(eyre!("{fname}: parsing failed: {e:?}"));
            }
        },
    };
    let sum = machines.iter().try_fold(0usize, |acc, m| {
        shortest_buttons(m)
            .map(|c| acc + c)
            .ok_or_eyre("did not find starting sequence")
    })?;
    println!("{sum}");
    let second = machines
        .iter()
        .enumerate() /*.skip(49).take(1)*/
        .try_fold(0usize, |acc, (i, m)| {
            //println!("{i}");
            shortest_joltages(m)
                .map(|c| {
                    println!("{i}: {c}");
                    acc + c
                })
                .ok_or_else(|| eyre!("{i}: did not find joltage sequence"))
        })?;
    println!("{second}");
    Ok(())
}
