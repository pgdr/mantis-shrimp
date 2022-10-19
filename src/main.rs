#![allow(non_snake_case)]
use graphbench::editgraph::EditGraph;
use graphbench::graph::*;
use graphbench::io::*;

use graphbench::ordgraph::*;
use itertools::*;

use fxhash::{FxHashMap, FxHashSet};

fn main() {

    let mut graph = EditGraph::from_gzipped("Yeast.txt.gz").expect("File not found");
    graph.remove_loops();

    let graph = OrdGraph::by_degeneracy(&graph);

    let mut R:FxHashMap<Vec<Vertex>, u32> = FxHashMap::default();

    for u in graph.vertices() {
        let N = graph.left_neighbours(u);

        for subset in N.into_iter().powerset() {
            R.entry(subset).and_modify(|c| *c += 1).or_insert(1);
        }
    }

    for (X, count) in R.iter() {
        println!("{X:?} -> {count}");
    }
}
