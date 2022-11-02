#![allow(non_snake_case)]
use std::collections::BTreeSet;

use graphbench::editgraph::EditGraph;
use graphbench::graph::*;
use graphbench::io::*;

use graphbench::degengraph::*;
use itertools::*;

use fxhash::{FxHashMap, FxHashSet};

fn main() {
    let mut graph = EditGraph::from_gzipped("Yeast.txt.gz").expect("File not found");   
    graph.remove_loops();
    let nquery = NQuery::new(&graph);

}

struct NQuery{
    R:FxHashMap<BTreeSet<Vertex>, u32>,
    graph:DegenGraph
}

impl NQuery {
    fn new(graph: &EditGraph) -> Self{
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

    fn is_shattered(&self, S: Vec<Vertex>) -> bool{
        unimplemented!()
        // loop over all subsets of S
        // run query_uncor for each subset
        // store the result in a HashMap 
        // (then we will apply the correction)
    }

    fn query_uncor(&self, X: BTreeSet<Vertex>, S: BTreeSet<Vertex>) -> i32{
        let S_minus_X:BTreeSet<_> = S.difference(&X).collect();
        let mut res:i32 = 0;

        for subset in S_minus_X.into_iter().powerset() {
            let subset: BTreeSet<u32> = subset.into_iter().cloned().collect();
            let Y:BTreeSet<u32> = X.union(&subset).cloned().collect();

            if subset.len() % 2 == 0{
                res += *self.R.get(&Y).unwrap_or(&0) as i32;
            } else{
                res -= *self.R.get(&Y).unwrap_or(&0) as i32;
            }
        }
        // to do: add correction.
        res
    } 
}