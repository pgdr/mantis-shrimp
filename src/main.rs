#![allow(non_snake_case)]
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
    R:FxHashMap<Vec<Vertex>, u32>,
    graph:DegenGraph
}

impl NQuery {
    fn new(graph: &EditGraph) -> Self{

        let graph = DegenGraph::from_graph(graph);    

        let mut R:FxHashMap<Vec<Vertex>, u32> = FxHashMap::default();

        for u in graph.vertices() {
            let N = graph.left_neighbours(u);

            for subset in N.into_iter().powerset() {

                R.entry(subset).and_modify(|c| *c += 1).or_insert(1);

            }
        }
        NQuery { R, graph }
    }

    fn is_shattered(&self, S: Vec<Vertex>) -> bool{
        unimplemented!()
    }

    fn query(&self, X: FxHashSet<Vertex>, S: FxHashSet<Vertex>) -> u32{

        let S_minus_X:FxHashSet<_> = S.difference(&X).collect();

        for subset in S_minus_X.into_iter().powerset() {

            let subset: FxHashSet<_> = subset.iter().cloned().collect();

            let Y = X.union(&subset);

        }
    }
}