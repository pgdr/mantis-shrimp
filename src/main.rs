#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]

mod algorithms;
mod io;
mod nquery;
mod setfunc;
mod skipcombs;
mod vecset;

// use std::backtrace::Backtrace;
use algorithms::*;
use io::load_graph;
use nquery::*;
use std::collections::BTreeSet;
use std::default;

use graphbench::editgraph::EditGraph;
use graphbench::graph::*;
use graphbench::io::load_vertex_set;

use graphbench::degengraph::*;
use itertools::*;

use fxhash::FxHashMap;

use clap::{Parser, ValueEnum};
use std::path::Path;

#[derive(Parser, Debug)]
#[clap(author, version="1.0", about, long_about = None)]
struct Args {
    #[clap(short, long)]
    help: bool,

    /// The statistic to compute
    #[clap(value_enum)]
    statistic: StatisticArg,

    /// The network file
    file: String,

    ///  (VC only) restrict search of shattered set to these vertices
    shattered_candidates: Option<String>,
}

#[derive(Clone, Debug, ValueEnum)]
enum StatisticArg {
    VC,
    Ladder,
    Crown,
    Biclique,
    CClosure,
}

fn main() -> Result<(), &'static str> {
    let args = Args::parse();
    let filename = args.file;

    // Load graph
    let path = Path::new(&filename);
    let mut graph = match load_graph(path) {
        Ok(G) => G,
        Err(msg) => {
            println!("{msg}");
            return Err("Parsing error");
        }
    };

    println!(
        "Loaded graph with n={} and m={}",
        graph.num_vertices(),
        graph.num_edges()
    );

    graph.remove_loops();
    let graph = DegenGraph::from_graph(&graph);

    let d = *graph.left_degrees().values().max().unwrap() as usize;
    let logd = (d as f32).log2();
    println!(
        "Computed degeneracy ordering with d={} (log d = {:.2})",
        d, logd
    );

    match args.statistic {
        StatisticArg::VC => {
            println!("Computing VC dimension");
            let mut alg = VCAlgorithm::new(&graph);

            if let Some(filename) = args.shattered_candidates {
                let cand_set = match load_vertex_set(&filename) {
                    Ok(cand_set) => cand_set,
                    Err(error) => {
                        println!("{:?}", error);
                        return Err("Could not parse candidate vertex set");
                    }
                };
                let cand_size = cand_set.len();
                println!("Restricting VC search to {cand_size} vertices contained in `{filename}`");
                alg.set_shatter_candidates(&cand_set);
            }

            alg.run();
        }
        StatisticArg::Ladder => {
            println!("Approximating ladder index");
            let mut alg = LadderAlgorithm::new(&graph);
            alg.run();
        }
        StatisticArg::Crown => {
            println!("Approximating crown size");
            let mut alg = CrownAlgorithm::new(&graph);
            alg.run();
        }
        StatisticArg::Biclique => {
            println!("Computing biclique size");
            let mut alg = BicliqueAlgorithm::new(&graph);
            alg.run();
        }
        StatisticArg::CClosure => {
            println!("Computing c-closure");
            let mut alg = CClosureAlgorithm::new(&graph);
            alg.run();
        }
    }

    Ok(())
}
