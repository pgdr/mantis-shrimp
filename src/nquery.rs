use graphbench::graph::*;
use graphbench::degengraph::DegenGraph;

use std::collections::BTreeSet;
use fxhash::FxHashMap;

use itertools::*;

pub struct NQuery<'a> {
    R:FxHashMap<BTreeSet<Vertex>, u32>,
    max_query_size: usize,
    graph:&'a DegenGraph
}

impl<'a> NQuery<'a> {
    pub fn new(graph:&'a DegenGraph) -> Self {  
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

    pub fn is_shattered(&mut self, S: &BTreeSet<Vertex>) -> bool {
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

    pub fn degree_profile(&self, v:&Vertex) -> Vec<usize> {
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
    use super::*;    
    use graphbench::{editgraph::EditGraph, graph::MutableGraph};
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