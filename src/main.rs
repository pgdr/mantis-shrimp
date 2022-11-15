#![allow(non_snake_case)]
// use std::backtrace::Backtrace;
use std::collections::BTreeSet;
use std::default;

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

    fn is_shattered(&self, S: &BTreeSet<Vertex>) -> bool {
        let mut I:FxHashMap<_, _> = FxHashMap::default();

        for subset in S.iter().powerset() { 
            let subset: BTreeSet<_> = subset.into_iter().cloned().collect();
            let res = self.query_uncor(&subset, S);
            I.insert(subset, res);
        }

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

            if v_left_S.is_empty() {
                I.entry(v_right_S).and_modify(|c| *c += 1 ).or_insert(1);
            } else {
                let v_S: BTreeSet<Vertex> = v_left_S.union(&v_right_S).cloned().collect();
                I.entry(v_left_S).and_modify(|c| *c -= 1 ).or_insert(1);
                I.entry(v_S).and_modify(|c| *c += 1 ).or_insert(1);
            }

            /*
                Felix: 
                This line should make you suspicious: we are not modifying the key set of `I` at all, 
                so why should we need a copy? The issue here is that .keys() gives us an iterator to the
                keys, it is the wrong approach here.

                Have a look at `I.contains_key` instead.
            */
            // let subsets: BTreeSet<_> = I.keys().cloned().collect();

            /*
                Felix:
                Patrick, have a look at the paper write-up of the correction. What we need is 
                not the left/right neighbourhood of v, but the *intersections* of the left/right neighbourhood
                with S. 

                The only special case you should need to check here is if $N^-(v) \cap S$ is empty, because that
                entry will not exist (and we don't really need it). The entry-api allows you to insert a value if none
                exists, e.g.
                    I.entry(X).and_modify(|c| *c += 1 ).or_insert(1)
                Will either modify and existing value for the key `X` by incrementing it by one or insert a new value
                for it (here 1).

            if subsets.contains(&v_right_neigh) {
                if subsets.contains(&v_left_neigh) {
                    let new_subset: BTreeSet<_> = v_left_neigh.union(&v_right_neigh).cloned().collect();
                    I.entry(v_left_neigh).and_modify(|c| *c -= 1);
                    I.entry(new_subset).and_modify(|c| *c += 1);

                } else {
                    I.entry(v_right_neigh).and_modify(|c| *c += 1);
                }
                */
            }
            // now to check
        true
    }

    fn query_uncor(&self, X: &BTreeSet<Vertex>, S: &BTreeSet<Vertex>) -> i32 {
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
}