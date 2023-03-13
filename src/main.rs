#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]

mod io;
mod nquery;
mod algorithms;

// use std::backtrace::Backtrace;
use std::collections::BTreeSet;
use std::default;
use io::load_graph;
use nquery::*;
use algorithms::*;

use graphbench::editgraph::EditGraph;
use graphbench::graph::*;

use graphbench::degengraph::*;
use itertools::*;

use fxhash::FxHashMap;

use clap::{Parser, ValueEnum};
use std::path::Path;



/// Counts 'butterflies' (4-cycles) in sparse bipartite networks.
#[derive(Parser, Debug)]
#[clap(author, version="1.0", about, long_about = None)]
struct Args {
    #[clap(short, long)]
    help: bool,

    /// The network file
    file:String,    
}



fn main() -> Result<(), &'static str> {
    let args = Args::parse();
    let filename = args.file.clone();

    // Load graph
    let path = Path::new(&filename);
    let mut graph = match load_graph(path) {
        Ok(G) => G,
        Err(msg) => {
            println!("{msg}");
            return Err("Parsing error");
        }
    };

    println!("Loaded graph with n={} and m={}", graph.num_vertices(), graph.num_edges());
    
    graph.remove_loops();
    let graph = DegenGraph::from_graph(&graph);  

    let d = *graph.left_degrees().values().max().unwrap() as usize;
    let logd = (d as f32).log2();    
    println!("Computed degeneracy ordering with d={} (log d = {:.2})", d, logd);


    let mut alg = VCAlgorithm::new(&graph);
    alg.run();


    // let mut graph = EditGraph::from_gzipped("Yeast.txt.gz").expect("File not found");   
    // let n = graph.num_vertices();
    // let mut nquery = NQuery::new(&graph);
    
    // Phase 1: Linear scan
    // let mut k = 2;

    // let order:Vec<_> = graph.vertices().cloned().collect();

    // let mut improved = true;
    // while improved && k+1 <= d {
    //     improved = false;
    //     for &v in &order {
    //         let mut N = graph.left_neighbours(&v);
    //         N.push(v);

    //         for S in N.iter().combinations(k+1) {
    //             let S:BTreeSet<Vertex> = S.into_iter().cloned().collect();
    //             if nquery.is_shattered(&S) {
    //                 k = k + 1;
    //                 println!("Found shattered set of size {k}");                    
    //                 improved = true;
    //                 break;
    //             }
    //         }
    //     }
    // }

    // println!("Largest one-covered shattered set: {:?}", k);

    // Check which vertices are valid candidates for a shattered set 
    // of size k
    // let degree_profile = generate_degree_profile(k+1);
    // println!("{degree_profile:?}");

    // let mut witness_candidates:VertexSet = VertexSet::default();
    // for &v in graph.vertices() {
    //     let degrees = nquery.degree_profile(&v);
    //     if dominates_profile(&degrees, &degree_profile) {
    //         witness_candidates.insert(v);
    //     }
    // }

    // println!("Found {} out of {n} as witness candidates for {k}-shattered set", witness_candidates.len());

    // let mut cover_candidates:VertexSet = witness_candidates.iter().cloned().collect(); 
    // for &v in graph.vertices() {
    //     let mut covers = false;
    //     for u in graph.left_neighbours_slice(&v) {
    //         if witness_candidates.contains(u) {
    //             covers = true;
    //             break;
    //         }
    //     }
    //     if covers {
    //         cover_candidates.insert(v);
    //     }
    // }

    // println!("Found {} out of {n} as cover candidates for {k}-shattered set", cover_candidates.len());

    // Look for chunks of size k / log d
    Ok(())
}