#![allow(non_snake_case)]

mod io;

// use std::backtrace::Backtrace;
use std::collections::BTreeSet;
use io::load_graph;

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

    // let mut graph = EditGraph::from_gzipped("Yeast.txt.gz").expect("File not found");   
    graph.remove_loops();
    let nquery = NQuery::new(&graph);

    let d = *nquery.graph.left_degrees().values().max().unwrap();
    let p = ((d as f64).log2() + 1.0).floor() as u32;

    println!("Degeneracy is d={d}");
    println!("Threshold is p={p}");
    
    /*
        Idea: Use bloom filter to store the k-sized shattered sets, then
        query to figure out whether a given (k+1)-sized set could be shattered
    */

    // Case 2 in paper: Shattered set is small
    let vertices:Vec<_> = graph.vertices().collect();
    for r in 3..=p { 
        println!("Searching sets of size r={r}");
        for S in vertices.iter().combinations(r as usize) {
            let S:BTreeSet<Vertex> = S.into_iter().cloned().cloned().collect();
            if nquery.is_shattered(&S) {
                println!("  Found shattered set for r={r}: {S:?}");
                break;
            }
        } 
    }

    // Case 1 in paper: Shattered set is large
    for r in (p+1)..(d+1) {
        // TODO
    }

    Ok(())
}

struct NQuery {
    R:FxHashMap<BTreeSet<Vertex>, u32>,
    graph:DegenGraph
}

impl NQuery {
    fn new(graph: &EditGraph) -> Self {
        let graph = DegenGraph::from_graph(graph);    
        let mut R:FxHashMap<_, _> = FxHashMap::default();

        for u in graph.vertices() {
            let N = graph.left_neighbours(u);

            for subset in N.into_iter().powerset() {
                R.entry(subset.into_iter().collect()).and_modify(|c| *c += 1).or_insert(1);
            }
        }
        NQuery { R, graph }
    }

    fn query_uncor(&self, X: &BTreeSet<Vertex>, S: &BTreeSet<Vertex>) -> i32 {
        if X.is_empty() {
            return 0
        }
        let S_minus_X:BTreeSet<_> = S.difference(&X).collect();
        let mut res:i32 = 0;

        for subset in S_minus_X.into_iter().powerset() {
            let subset: BTreeSet<u32> = subset.into_iter().cloned().collect();
            let Y:BTreeSet<u32> = X.union(&subset).cloned().collect();

            if subset.len() % 2 == 0 {
                res += *self.R.get(&Y).unwrap_or(&0) as i32;
            } else {
                res -= *self.R.get(&Y).unwrap_or(&0) as i32;
            }
        }
        res
    }

    fn left_neighbour_set(&self, S: &BTreeSet<Vertex>) -> BTreeSet<Vertex> {
        let mut res: BTreeSet<Vertex> = BTreeSet::default();

        for u in S {
            let l_neigh = self.graph.left_neighbours(u);
            res.extend(l_neigh.into_iter())
        }
        res  
    }

    fn is_shattered(&self, S: &BTreeSet<Vertex>) -> bool {
        let mut I:FxHashMap<_, _> = FxHashMap::default();
        
        let mut res_sum = 0;
        for subset in S.iter().powerset() { 
            let subset: BTreeSet<_> = subset.into_iter().cloned().collect();
            let res = self.query_uncor(&subset, S);
            res_sum += res;
            I.insert(subset, res);
        }
        I.insert(BTreeSet::default(), self.graph.num_vertices() as i32 - res_sum);

        // correction
        let left_neighs = self.left_neighbour_set(S); 

        for v in left_neighs {   
            // collect v's left and right neighbourhoods 
            let neigh: BTreeSet<Vertex> = self.graph.neighbours(&v).cloned().collect();
            let v_left_neigh: BTreeSet<Vertex> = self.graph.left_neighbours(&v).into_iter().collect();
            let v_right_neigh: BTreeSet<Vertex> = neigh.difference(&v_left_neigh).cloned().collect();
            
            // take the intersections of the neighbourhoods with S
            let v_left_S: BTreeSet<Vertex> = S.intersection(&v_left_neigh).cloned().collect();
            let v_right_S: BTreeSet<Vertex> = S.intersection(&v_right_neigh).cloned().collect();

            let v_S: BTreeSet<Vertex> = v_left_S.union(&v_right_S).cloned().collect();
            I.entry(v_left_S).and_modify(|c| *c -= 1 ).or_insert(1);
            I.entry(v_S).and_modify(|c| *c += 1 ).or_insert(1);
        }
        
        if I.len() != 2_usize.pow(S.len() as u32) {
            return false
        }

        for (_k, v) in I.iter() {
            if *v == 0 {
                return false
            }
        }
        return true
    }
}


#[cfg(test)]
mod  tests {
    use graphbench::{editgraph::EditGraph, graph::MutableGraph};
    use crate::NQuery;
    use std::collections::BTreeSet;

    #[test]
    fn shattered_test1 () {
        let mut graph = EditGraph::from_txt("test1_shattered.txt").expect("File not found.");
        graph.remove_loops();
        let nquery = NQuery::new(&graph);

        let sh_set = BTreeSet::from([1, 2, 3, 4]);
        let result = nquery.is_shattered(&sh_set);
        assert_eq!(result, true);
    }

    #[test]
    fn shattered_test2 () {
        let mut graph = EditGraph::from_txt("test1_shattered.txt").expect("File not found.");
        graph.remove_loops();
        let nquery = NQuery::new(&graph);

        let unsh_set = BTreeSet::from([1, 2, 3, 16]);
        let result = nquery.is_shattered(&unsh_set);
        assert_eq!(result, false);
    }
}