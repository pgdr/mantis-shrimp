use std::collections::BTreeSet;

use graphbench::graph::*;
use graphbench::degengraph::DegenGraph;

use crate::nquery::NQuery;

use itertools::*;

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

pub struct VCAlgorithm<'a> {
    graph: &'a DegenGraph,
    nquery: NQuery<'a>,
    witness_candidates:VertexSet,
    cover_candidates:VertexSet,
    vc_dim:usize,
    d: usize,
    logd: f32
}

impl<'a> VCAlgorithm<'a> {
    pub fn new(graph: &'a DegenGraph) -> Self {
        let d = *graph.left_degrees().values().max().unwrap() as usize;
        let logd = (d as f32).log2();    

        let witness_candidates:VertexSet = graph.vertices().cloned().collect();
        let mut cover_candidates:VertexSet = witness_candidates.iter().cloned().collect(); 

        let mut nquery = NQuery::new(&graph);
        VCAlgorithm{ graph, d, logd, witness_candidates, cover_candidates, nquery, vc_dim: 1}
    }

    pub fn run(&mut self) {
        let mut improved = true;
        while improved && self.vc_dim+1 <= self.d {
            improved = false;
            println!("Checking {} candidates", self.cover_candidates.len() );
            for &v in &self.cover_candidates {
                let mut N = self.graph.left_neighbours(&v);
                N.push(v);

                for S in N.iter().combinations(self.vc_dim+1) {
                    let S:BTreeSet<Vertex> = S.into_iter().cloned().collect();
                    if self.nquery.is_shattered(&S) {
                        self.vc_dim += 1;
                        println!("Found shattered set of size {}", self.vc_dim);                    
                        improved = true;
                        break;
                    }
                }

                if improved {
                    break;
                }
            }

            println!("? {improved}");
            if improved {
                self.recompute_candidates();
            }
        }

        println!("Largest one-covered shattered set: {:?}", self.vc_dim);
    }

    fn recompute_candidates(&mut self) {
        println!("Recomputing candidates");
        let degree_profile = generate_degree_profile(self.vc_dim+1);
        let n = self.graph.num_vertices();
        println!("{degree_profile:?}");

        self.witness_candidates.retain(|v| {
            let degrees = self.nquery.degree_profile(&v);
            dominates_profile(&degrees, &degree_profile)
        });

        println!("Found {} out of {n} as witness candidates for {}-shattered set", self.witness_candidates.len(), self.vc_dim);

        self.cover_candidates.retain(|v| {
            let mut covers = false;
            for u in self.graph.left_neighbours_slice(&v) {
                if self.witness_candidates.contains(u) {
                    covers = true;
                    break;
                }
            }
            covers
        });

        println!("Found {} out of {n} as cover candidates for {}-shattered set", self.cover_candidates.len(), self.vc_dim);
    }
}   

