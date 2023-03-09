#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]

mod io;

// use std::backtrace::Backtrace;
use std::collections::BTreeSet;
use std::default;
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
    let n = graph.num_vertices();
    let mut nquery = NQuery::new(graph);

    let d = *nquery.graph.left_degrees().values().max().unwrap() as usize;
    let logd = (d as f32).log2();

    println!("Degeneracy is d={d}");
    
    // Phase 1: Linear scan
    let mut k = 2;

    let order:Vec<_> = nquery.graph.vertices().cloned().collect();

    let mut improved = true;
    while improved && k+1 <= d {
        improved = false;
        for &v in &order {
            let mut N = nquery.graph.left_neighbours(&v);
            N.push(v);

            for S in N.iter().combinations(k+1) {
                let S:BTreeSet<Vertex> = S.into_iter().cloned().collect();
                if nquery.is_shattered(&S) {
                    k = k + 1;
                    println!("Found shattered set of size {k}");                    
                    improved = true;
                    break;
                }
            }
        }
    }

    println!("Largest one-covered shattered set: {:?}", k);

    // Check which vertices are valid candidates for a shattered set 
    // of size k
    let degree_profile = generate_degree_profile(k+1);
    // println!("{degree_profile:?}");

    let mut witness_candidates:VertexSet = VertexSet::default();
    for &v in nquery.graph.vertices() {
        let degrees = nquery.degree_profile(&v);
        if dominates_profile(&degrees, &degree_profile) {
            witness_candidates.insert(v);
        }
    }

    println!("Found {} out of {n} as witness candidates for {k}-shattered set", witness_candidates.len());

    let mut cover_candidates:VertexSet = witness_candidates.iter().cloned().collect(); 
    for &v in nquery.graph.vertices() {
        let mut covers = false;
        for u in nquery.graph.left_neighbours_slice(&v) {
            if witness_candidates.contains(u) {
                covers = true;
                break;
            }
        }
        if covers {
            cover_candidates.insert(v);
        }
    }

    println!("Found {} out of {n} as cover candidates for {k}-shattered set", cover_candidates.len());

    // Look for chunks of size k / log d
    Ok(())
}

fn binom(n: usize, k: usize) -> usize {
    let mut res = 1;
    for i in 0..k {
        res = (res * (n - i)) / (i + 1);
    }
    res
}

fn generate_degree_profile(k:usize) -> Vec<usize> {
    let mut res = Vec::default();
    for d in (1..=k).rev() {
        for _ in 0..binom(k, d) {
            res.push(d);
        }
    }
    res
}

fn dominates_profile(degA:&Vec<usize>, degB:&Vec<usize>) -> bool {
    if degA.len() < degB.len() {
        return false;
    }

    for (dA,dB) in degA.iter().zip(degB.iter()) {
        if dA < dB {
            return false;
        }
    }

    return true;
}

struct NQuery {
    R:FxHashMap<BTreeSet<Vertex>, u32>,
    max_query_size: usize,
    graph:DegenGraph
}

impl NQuery {
    fn new(graph:EditGraph) -> Self {
        let graph = DegenGraph::from_graph(&graph);    
        let mut R:FxHashMap<_, _> = FxHashMap::default();

        let mut res = NQuery { R, graph, max_query_size: 0 };
        res.ensure_size(3);

        res        
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

    fn ensure_size(&mut self, size:usize) {
        if size <= self.max_query_size {
            return;
        }

        println!("Recomputing R for query size {size}...");

        for s in (self.max_query_size+1)..=size {
            for u in self.graph.vertices() {
                let N = self.graph.left_neighbours(u);
    
                for subset in N.into_iter().combinations(s) {
                    self.R.entry(subset.into_iter().collect()).and_modify(|c| *c += 1).or_insert(1);
                }
            }
        }

        self.max_query_size = size;
    }

    fn is_shattered(&mut self, S: &BTreeSet<Vertex>) -> bool {
        let mut I:FxHashMap<_, _> = FxHashMap::default();
        
        self.ensure_size(S.len());

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

    fn degree_profile(&self, v:&Vertex) -> Vec<usize> {
        let mut degrees = Vec::default();
        for u in self.graph.neighbours(&v) {
            degrees.push(self.graph.degree(u) as usize);
        }
        degrees.sort_unstable();
        degrees.reverse();
        degrees
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
        let mut nquery = NQuery::new(graph);

        let sh_set = BTreeSet::from([1, 2, 3, 4]);
        let result = nquery.is_shattered(&sh_set);
        assert_eq!(result, true);
    }

    #[test]
    fn shattered_test2 () {
        let mut graph = EditGraph::from_txt("test1_shattered.txt").expect("File not found.");
        graph.remove_loops();
        let mut nquery = NQuery::new(graph);

        let unsh_set = BTreeSet::from([1, 2, 3, 16]);
        let result = nquery.is_shattered(&unsh_set);
        assert_eq!(result, false);
    }
}