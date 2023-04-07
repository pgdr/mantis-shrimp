use graphbench::graph::*;
use graphbench::degengraph::DegenGraph;

use std::collections::BTreeSet;
use fxhash::FxHashMap;

use itertools::*;

use crate::{setfunc::{SetFunc, SmallSetFunc}, vecset::{difference, union, intersection}};

pub struct NQuery<'a> {
    R:SetFunc,
    max_query_size: usize,
    graph:&'a DegenGraph
}

impl<'a> NQuery<'a> {
    pub fn new(graph:&'a DegenGraph) -> Self {  
        let mut R = SetFunc::default();

        NQuery { R, graph, max_query_size: 0 }
    }

    fn query_uncor(&self, X: &Vec<Vertex>, S: &Vec<Vertex>) -> i32 {
        if X.is_empty() {
            return 0
        }

        let S_minus_X:Vec<_> = difference(S, X);
        let mut res:i32 = 0;

        for subset in S_minus_X.into_iter().powerset() {
            let Y = union(X, &subset);

            if subset.len() % 2 == 0 {
                res += self.R[&Y];
            } else {
                res -= self.R[&Y];
            }
        }
        res
    }

    fn fast_mobius(&self, S: &BTreeSet<Vertex>) -> FxHashMap<BTreeSet<Vertex>, i32> {
        

        todo!()
    }

    fn left_neighbour_set(&self, S: &Vec<Vertex>) -> Vec<Vertex> {
        let mut res: BTreeSet<Vertex> = BTreeSet::default();

        for u in S {
            let l_neigh = self.graph.left_neighbours(u);
            res.extend(l_neigh.into_iter())
        }
    
        res.into_iter().collect()
    }

    pub fn ensure_size(&mut self, size:usize, query_candidates:&VertexSet) {
        if size <= self.max_query_size {
            return;
        }

        println!("Recomputing R for query size {size}...");

        for s in (self.max_query_size+1)..=size {
            for u in self.graph.vertices() {
                let mut N = self.graph.left_neighbours(u);
                N.retain(|x| query_candidates.contains(x));
                N.sort_unstable();
    
                for subset in N.into_iter().combinations(s) {
                    self.R[&subset] += 1;
                }
            }
        }

        self.max_query_size = size;
    }

    pub fn is_shattered(&mut self, S: &Vec<Vertex>) -> bool {
        let mut S:Vec<u32> = S.iter().cloned().collect();
        S.sort_unstable();

        println!("{S:?}");

        let mut I = SmallSetFunc::new(&S);
        
        assert!(S.len() <= self.max_query_size);

        let mut res_sum = 0;
        for subset in S.iter().powerset() { 
            let subset:Vec<u32> = subset.into_iter().cloned().collect();
            let res = self.query_uncor(&subset, &S);
            res_sum += res;
            I[&subset] = res;
        }

        // Insert correct value for the empty set manually
        I[&vec![]] = self.graph.num_vertices() as i32 - res_sum;
        println!("{I}");

        // correction
        let left_neighs = self.left_neighbour_set(&S);
        println!("Left-neighbours = {left_neighs:?}");

        for v in left_neighs {   
            // Collect v's left and right neighbourhoods 
            let N: Vec<Vertex> = self.graph.neighbours(&v).cloned().sorted_unstable().collect();
            println!("{v} has neighbours {N:?}");
            let N_left: Vec<Vertex> = self.graph.left_neighbours(&v).into_iter().sorted_unstable().collect();
            let N_right = difference(&N, &N_left);
            
            // Take the intersections of the neighbourhoods with S
            let N_left = intersection(&S, &N_left);
            let N_right = intersection(&S, &N_right);

            let N = union(&N_left, &N_right);

            debug_assert!(I[&N_left] > 0);
            println!("Move {v} from {N_left:?} to {N:?}");
            I[&N_left] -= 1;
            I[&N] += 1;
        }
        println!("{I}");
        
        if I.count_nonzero() != 2_usize.pow(S.len() as u32) {
            return false
        }
        true
    }

    pub fn degree_profile(&self, v:&Vertex) -> Vec<usize> {
        let mut degrees = Vec::default();
        for u in self.graph.neighbours(v) {
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
    use rand::prelude::*;
    use std::collections::BTreeSet;

    #[test]
    fn shattered_test1 () {
        let mut graph = EditGraph::from_txt("test1_shattered.txt").expect("File not found.");
        let graph = DegenGraph::with_ordering(&graph, vec![1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16].iter());  

        let mut nquery = NQuery::new(&graph);
        nquery.ensure_size(4, &graph.vertices().cloned().collect());

        let sh_set = vec![1, 2, 3, 4];
        let result = nquery.is_shattered(&sh_set);
        assert_eq!(result, true);
    }

    #[test]
    fn shattered_test2 () {
        let mut graph = EditGraph::from_txt("test1_shattered.txt").expect("File not found.");
        let graph = DegenGraph::from_graph(&graph);  

        let mut nquery = NQuery::new(&graph);
        nquery.ensure_size(4, &graph.vertices().cloned().collect());

        let unsh_set = vec![1, 2, 3, 16];
        let result = nquery.is_shattered(&unsh_set);
        assert_eq!(result, false);
    }

    #[test]
    fn shattered_test_small() {
        let mut rng = rand::thread_rng();
        for k in 2..=5 {
            let mut G = EditGraph::new();
            G.add_vertices((0..k).into_iter());
            
            let mut v = k;
            let mut order = (0..k).into_iter().collect_vec();
            for set in (0..k).into_iter().powerset() {
                println!("{v} -> {set:?}");
                G.add_vertex(&v);
                for u in set {
                    G.add_edge(&u, &v);
                }
                order.push(v);
                v += 1;
            }

            let D = DegenGraph::with_ordering(&G, order.iter());
            let mut nquery = NQuery::new(&D);
            nquery.ensure_size(k as usize, &D.vertices().cloned().collect());

            let result = nquery.is_shattered(&(0..k).into_iter().collect_vec());
            assert_eq!(result, true);

            order.shuffle(&mut rng);
            println!("{order:?}");
            let D = DegenGraph::with_ordering(&G, order.iter());
            let mut nquery = NQuery::new(&D);
            nquery.ensure_size(k as usize, &D.vertices().cloned().collect());

            let result = nquery.is_shattered(&(0..k).into_iter().collect_vec());
            assert_eq!(result, true);            
        }
    }

    #[test]
    fn bad() {
        let k = 3;
        let mut G = EditGraph::new();
        G.add_vertices((0..k).into_iter());
        
        let mut v = k;
        for set in (0..k).into_iter().powerset() {
            println!("{v} -> {set:?}");
            G.add_vertex(&v);
            for u in set {
                G.add_edge(&u, &v);
            }
            v += 1;
        }

        let order = vec![1,3,9,0,6,7,10,2,5,4,8];
        let D = DegenGraph::with_ordering(&G, order.iter());
        let mut nquery = NQuery::new(&D);
        nquery.ensure_size(k as usize, &D.vertices().cloned().collect());

        let result = nquery.is_shattered(&(0..k).into_iter().collect_vec());
        assert_eq!(result, true);
    }    
}