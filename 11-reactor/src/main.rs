use std::collections::{HashMap, VecDeque};

use eyre::{OptionExt, eyre};
use fxhash::FxBuildHasher;
use nom::bytes::complete::{tag, take_till1};
use nom::character::complete::{char, newline};
use nom::combinator::{all_consuming, opt};
use nom::multi::separated_list1;
use nom::sequence::terminated;
use nom::{IResult, Parser};
use petgraph::algo::all_simple_paths;
use petgraph::graphmap::DiGraphMap;

// fn parse_name(delimiter: char) -> impl Fn(&str) -> IResult<&str,&str> {
//     move |input: &str| {
//         let mut buffer = [0u8; 4];
//         let tag: &str = delimiter.encode_utf8(&mut buffer);
//         //map(take_until1(tag), |s: &str| s.to_string()).parse(input)
//         take_until1(tag).parse(input)
//     }
// }

fn parse_name(stops: &str) -> impl Fn(&str) -> IResult<&str, &str> {
    move |input: &str| take_till1(|c| stops.contains(c)).parse(input)
}

fn parse_device(input: &str) -> IResult<&str, (&str, Vec<&str>)> {
    let res = (
        terminated(parse_name(":"), tag(": ")),
        separated_list1(char(' '), parse_name(" \n")),
    )
        .parse(input)?;
    Ok(res)
}

fn parse_file(input: &str) -> IResult<&str, Vec<(&str, Vec<&str>)>> {
    let mut parser =
        all_consuming(terminated(separated_list1(newline, parse_device), opt(newline)));
    parser.parse(input)
}

fn make_graph<'a>(devices: &Vec<(&'a str, Vec<&'a str>)>) -> DiGraphMap<&'a str, ()> {
    let num_nodes = devices.len();
    let mut graph = DiGraphMap::with_capacity(num_nodes, 8 * num_nodes);
    for (src, dests) in devices {
        for dest in dests {
            graph.add_edge(*src, *dest, ());
        }
    }
    graph
}

fn reverse_paths<'a>(
    devices: &Vec<(&'a str, Vec<&'a str>)>,
) -> HashMap<&'a str, Vec<&'a str>, FxBuildHasher> {
    let mut result: HashMap<&str, Vec<&str>, FxBuildHasher> =
        HashMap::with_capacity_and_hasher(devices.len(), FxBuildHasher::new());
    for (input, outputs) in devices {
        for output in outputs {
            result.entry(*output).or_default().push(*input);
        }
    }
    result
}

fn make_limited_graph<'a, S>(
    devices: &HashMap<&'a str, Vec<&'a str>, S>,
    from: &'a str,
    to: &'a str,
) -> DiGraphMap<&'a str, ()>
where
    S: std::hash::BuildHasher,
{
    let num_nodes = devices.len();
    let mut graph = DiGraphMap::with_capacity(num_nodes, 8 * num_nodes);
    let mut queue = VecDeque::from([to]);
    while let Some(node) = queue.pop_front() {
        if node != from
            && let Some(sources) = devices.get(node)
        {
            for &source in sources {
                if !graph.contains_node(source) {
                    queue.push_back(source);
                }
                graph.add_edge(source, node, ());
            }
        }
    }
    graph
}

fn count_limited<S>(
    devices: &HashMap<&str, Vec<&str>, S>,
    from: &str,
    to: &str,
) -> usize
where
    S: std::hash::BuildHasher,
{
    let graph = make_limited_graph(devices, from, to);
    if graph.contains_node(from) {
        all_simple_paths::<Vec<_>, _, FxBuildHasher>(&graph, from, to, 0, None).count()
    } else {
        0
    }
}

fn dsp_paths<S>(devices: &HashMap<&str, Vec<&str>, S>) -> usize
where
    S: std::hash::BuildHasher,
{
    let svr_fft_out = {
        let fft_dac = count_limited(devices, "fft", "dac");
        if fft_dac > 0 {
            let svr_fft = count_limited(devices, "svr", "fft");
            if svr_fft > 0 {
                let dac_out = count_limited(devices, "dac", "out");
                dac_out * svr_fft * fft_dac
            } else {
                0
            }
        } else {
            0
        }
    };
    let svr_dac_out = {
        let dac_fft = count_limited(devices, "dac", "fft");
        if dac_fft > 0 {
            let svr_dac = count_limited(devices, "svr", "dac");
            if svr_dac > 0 {
                let fft_out = count_limited(devices, "fft", "out");
                dac_fft * svr_dac * fft_out
            } else {
                0
            }
        } else {
            0
        }
    };
    svr_fft_out + svr_dac_out
}

fn main() -> eyre::Result<()> {
    let mut args = std::env::args();
    let fname = args.nth(1).ok_or_eyre("filename was not provided")?;
    let body: String = std::fs::read_to_string(&fname)?;
    let devices = match parse_file(&body) {
        Ok((_, v)) => v,
        Err(e) => match e {
            nom::Err::Incomplete(_) => unreachable!(),
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                return Err(eyre!("{fname}: parsing failed: {e:?}"));
            }
        },
    };
    let graph = make_graph(&devices);
    let you_out_count = all_simple_paths::<Vec<_>, _, fxhash::FxBuildHasher>(
        &graph, "you", "out", 0, None,
    )
    .count();
    println!("{you_out_count}");
    let reversed = reverse_paths(&devices);
    println!("{}", dsp_paths(&reversed));
    Ok(())
}
