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

    true
}

pub struct VCAlgorithm<'a> {
    graph: &'a DegenGraph,
    nquery: NQuery<'a>,
    local_lower_bound:VertexMap<u8>,
    local_upper_bound:VertexMap<u8>,
    shatter_candidates:VertexSet,
    cover_candidates:VertexSet,
    vc_dim:usize,
    d: usize,
    logd: f32
}

impl<'a> VCAlgorithm<'a> {
    pub fn new(graph: &'a DegenGraph) -> Self {
        let d = *graph.left_degrees().values().max().unwrap() as usize;
        let logd = (d as f32).log2();    

        let shatter_candidates:VertexSet = graph.vertices().cloned().collect();
        let mut cover_candidates:VertexSet = shatter_candidates.iter().cloned().collect(); 

        let local_lower_bound = VertexMap::default();
        let local_upper_bound = graph.left_degrees().iter().map(|(k,v)| (*k,1 + *v as u8)).collect();

        let vc_dim = 1;
        let mut nquery = NQuery::new(graph);
        VCAlgorithm{ graph, d, logd, shatter_candidates, cover_candidates, nquery, local_lower_bound, local_upper_bound, vc_dim}
    }

    pub fn run(&mut self) {
        let mut improved = true;
        let mut cover_size = 1;

        while improved && self.vc_dim <= self.d+1 {
            improved = false;

            let brute_force_estimate = binom(self.shatter_candidates.len(), self.vc_dim+1);
            let cover_estimate = binom(self.cover_candidates.len(), cover_size) * binom(cover_size * self.d, self.vc_dim+1);

            self.nquery.ensure_size(self.vc_dim+1, &self.shatter_candidates);

            if brute_force_estimate < cover_estimate {
                println!("Brute-force: ({} choose {}) candidates", self.shatter_candidates.len(), self.vc_dim+1 );        
                // Test each subset of size vc_dim+1 whether it is shattered
                if self.vc_dim <= 3 {
                    for S in self.shatter_candidates.iter().combinations(self.vc_dim+1) {
                        let S:Vec<u32> = S.into_iter().cloned().collect(); // TODO: Better way of converting Vec<&u32> to Vec<u32>?
                        
                        if self.nquery.is_shattered(&S) {
                            self.vc_dim += 1;
                            println!("Found shattered set of size {}", self.vc_dim);                    
                            improved = true;
                            break;
                        }                                            
                    }
                } else {
                    let mut taboo = vec![u32::MAX; 3];
                    for S in self.shatter_candidates.iter().combinations(self.vc_dim+1) {
                        let S:Vec<u32> = S.into_iter().cloned().collect(); // TODO: Better way of converting Vec<&u32> to Vec<u32>?
                        
                        if S[..3] == taboo {
                            continue; 
                        }

                        if !self.nquery.is_shattered(&S[..3]) {
                            taboo.copy_from_slice(&S[..3]); // New taboo prefix
                            continue
                        }
                        taboo = vec![u32::MAX; 3];

                        if self.nquery.is_shattered(&S) {
                            self.vc_dim += 1;
                            println!("Found shattered set of size {}", self.vc_dim);                    
                            improved = true;
                            break;
                        }                                            
                    }                    
                }

                if !improved {
                    break; // No further improvement possible
                }
            } else if cover_size == 1 {
                println!("Covering: {} candidates", self.cover_candidates.len() );
                'outer: for c in self.cover_candidates.iter() {
                    if self.local_upper_bound[c] as usize <= self.vc_dim {
                        continue;
                    }

                    // Collect candidate set
                    let mut N:VertexSet = VertexSet::default();
                    N.insert(*c);
                    N.extend( self.graph.left_neighbours_slice(c));

                    // Retain only those elements that are witness candidates
                    N.retain(|x| self.shatter_candidates.contains(x) );

                    if N.len() <= self.vc_dim {
                        continue;
                    }

                    // Test each subset of size vc_dim+1 whether it is shattered
                    // println!("  Checking ({} choose {}) subsets for cover {:?}", N.len(), self.vc_dim+1, C);
                    for S in N.iter().combinations(self.vc_dim+1) {
                        let S = S.into_iter().cloned().collect_vec();
                        if self.nquery.is_shattered(&S) {
                            self.vc_dim += 1;
                            println!("Found shattered set of size {}", self.vc_dim);                    

                            // Update local lower bound on vc dimension
                            self.local_lower_bound.insert(*c, self.vc_dim as u8);

                            improved = true;
                            break 'outer;
                        }
                    }

                    // We exhaustively searched this vertex' neighbourhood, so we know
                    // it does not contain shattered sets of size vc_dim+1.
                    self.local_upper_bound.entry(*c).and_modify(|e| *e = std::cmp::min(*e, self.vc_dim as u8));
                }                    
            } else if cover_size <= 4 {
                println!("Covering: ({} choose {}) candidates", self.cover_candidates.len(), cover_size );
                'outer: for C in self.cover_candidates.iter().combinations(cover_size) {
                    let joint_upper_bound:usize = C.iter().map(|u| self.local_upper_bound[u] as usize).sum(); 
                    if joint_upper_bound <= self.vc_dim {
                        continue;
                    }

                    // Collect candidate set
                    let mut N:VertexSet = C.iter().map(|u| **u).collect();
                    for &u in &C {
                        N.extend( self.graph.left_neighbours_slice(u));
                    }

                    // Retain only those elements that are witness candidates
                    N.retain(|x| self.shatter_candidates.contains(x) );

                    if N.len() <= self.vc_dim {
                        continue;
                    }

                    // Test each subset of size vc_dim+1 whether it is shattered
                    // println!("  Checking ({} choose {}) subsets for cover {:?}", N.len(), self.vc_dim+1, C);
                    for S in N.iter().combinations(self.vc_dim+1) {
                        let S = S.into_iter().cloned().collect_vec();

                        if self.nquery.is_shattered(&S) {
                            self.vc_dim += 1;
                            println!("Found shattered set of size {}", self.vc_dim);                    
                            improved = true;
                            break 'outer;
                        }
                    }
                }                
            } else {
                println!("Covering: ({} choose {}) candidates", self.cover_candidates.len(), cover_size );
                'outer: for C in self.cover_candidates.iter().combinations(cover_size) {
                    let joint_upper_bound:usize = C.iter().map(|u| self.local_upper_bound[u] as usize).sum(); 
                    if joint_upper_bound <= self.vc_dim {
                        continue;
                    }

                    // Collect candidate set
                    let mut N:VertexSet = C.iter().map(|u| **u).collect();
                    for &u in &C {
                        N.extend( self.graph.left_neighbours_slice(u));
                    }

                    // Retain only those elements that are witness candidates
                    N.retain(|x| self.shatter_candidates.contains(x) );

                    if N.len() <= self.vc_dim {
                        continue;
                    }

                    // Test each subset of size vc_dim+1 whether it is shattered
                    let mut taboo = vec![u32::MAX; 3];
                    for S in N.iter().combinations(self.vc_dim+1) {
                        let S:Vec<u32> = S.into_iter().cloned().collect();
                        if S[..3] == taboo {
                            continue; 
                        }

                        if !self.nquery.is_shattered(&S[..3]) {
                            taboo.copy_from_slice(&S[..3]); // New taboo prefix
                            continue
                        }
                        taboo = vec![u32::MAX; 3];

                        if self.nquery.is_shattered(&S) {
                            self.vc_dim += 1;
                            println!("Found shattered set of size {}", self.vc_dim);                    
                            improved = true;
                            break 'outer;
                        }
                    }
                }              
            }

            if improved {
                println!("Found larger set, recomputing ");
                self.recompute_candidates();

                if self.shatter_candidates.len() <= self.vc_dim {
                    break;  // No further improvement possible
                }
            } else if cover_size < self.logd.ceil() as usize {
                improved = true;
                cover_size += 1;
                println!("No improvement, increasing cover size to {cover_size}");
            }
        }

        println!("Largest shattered set: {:?}", self.vc_dim);
    }

    fn recompute_candidates(&mut self) {
        println!("  > Recomputing candidates");
        let degree_profile = generate_degree_profile(self.vc_dim+1);
        let n = self.graph.num_vertices();
        println!("  > Degree profile is {degree_profile:?}");

        self.shatter_candidates.retain(|v| {
            let degrees = self.nquery.degree_profile(v);
            dominates_profile(&degrees, &degree_profile)
        });

        println!("  > Found {} out of {n} as witness candidates for {}-shattered set", self.shatter_candidates.len(), self.vc_dim);

        self.cover_candidates.retain(|v| {
            let mut covers = false;

            let mut num_cands = self.graph.left_neighbours_slice(v).iter()
                .filter(|u| self.shatter_candidates.contains(u)).count();
            num_cands += self.shatter_candidates.contains(v) as usize;

            // Update local upper bound
            self.local_upper_bound.entry(*v).and_modify(|e| *e = std::cmp::min(*e, num_cands as u8));

            // v only remains a cover candidate if it sees at least one 
            num_cands > 0
        });

        println!("  > Found {} out of {n} as cover candidates for {}-shattered set", self.cover_candidates.len(), self.vc_dim);
    }
}   

