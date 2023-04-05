#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_snake_case)]

use core::fmt;
use std::ops::{Index, IndexMut, Add, Sub};

use fxhash::FxHashMap;

#[derive(Debug)]
struct SmallSetFunc {
    universe:Vec<u32>,
    index_map:FxHashMap<u32, u8>,
    values:FxHashMap<u128, i32>    
}

impl SmallSetFunc {
    pub fn new<'a, I: IntoIterator<Item=&'a u32>>(universe:I) -> Self {
        let mut universe:Vec<u32> = universe.into_iter().cloned().collect();
        universe.sort_unstable();
        universe.dedup();

        Self::from_sorted_vec(universe)
    }

    fn from_sorted_vec(universe:Vec<u32>) -> Self {
        let index_map = universe.iter().enumerate().map(|(i,v)| (*v, i as u8) ).collect();
        let values = FxHashMap::default();

        SmallSetFunc{ universe, index_map, values }
    }

    fn convert_set<'a, I: IntoIterator<Item=&'a u32>>(&self, set:I) -> u128 {
        let mut res = 0u128;
        for x in set.into_iter() {
            let ix = self.index_map[x];
            assert!(ix < 128);
            res |= 1 << ix;
        }
        res
    }

    fn convert_bitset(&self, bitset:u128) -> Vec<u32> {
        let mut bitset = bitset;
        let mut res:Vec<u32> = Vec::default();
        while bitset != 0 {
            let ix = u128::trailing_zeros(bitset);
            bitset ^= 1 << ix;
            res.push(self.universe[ix as usize]);
        }

        res
    }

    pub fn entries_nonzero(&self) -> impl Iterator<Item=(Vec<u32>, i32)> + '_ {
        let res = self.values.iter()
            .filter(|(bitset, value)| **value != 0)
            .map(|(bitset,value)| (self.convert_bitset(*bitset), *value) );
        res
    }

    pub fn keys_nonzero(&self) -> impl Iterator<Item=Vec<u32>> + '_ {
        let res = self.values.iter()
            .filter(|(_, value)| **value != 0)
            .map(|(bitset,_)| (self.convert_bitset(*bitset)) );
        res
    }   

    pub fn count_nonzero(&self) -> usize {
        self.values.iter().filter(|(_, value)| **value != 0).count()
    } 
}

impl<'a, I> Index<I> for SmallSetFunc where I: IntoIterator<Item=&'a u32> {
    type Output = i32;

    fn index(&self, query: I) -> &Self::Output {
        let bitset = self.convert_set(query);
        if self.values.contains_key(&bitset) {
            &self.values[&bitset]
        } else {
            &0
        }
    }
}

impl<'a, I> IndexMut<I> for SmallSetFunc where I: IntoIterator<Item=&'a u32> {
    fn index_mut(&mut self, query: I) -> &mut Self::Output {
        let bitset = self.convert_set(query);
        self.values.entry(bitset).or_default()
    }
}

impl Add<SmallSetFunc> for SmallSetFunc {
    type Output = SmallSetFunc;

    fn add(self, rhs: SmallSetFunc) -> Self::Output {
        assert_eq!(self.universe, rhs.universe);
        // Since both functions have the same universe, they also have the same
        // bitset representations.         

        let mut res = SmallSetFunc::from_sorted_vec(self.universe.clone());
        for (bitset, value) in self.values.iter().chain(rhs.values.iter()) {
            *res.values.entry(*bitset).or_default() += value;            
        }

        res
    }
}

impl Sub<SmallSetFunc> for SmallSetFunc {
    type Output = SmallSetFunc;

    fn sub(self, rhs: SmallSetFunc) -> Self::Output {
        assert_eq!(self.universe, rhs.universe);
        // Since both functions have the same universe, they also have the same
        // bitset representations.         

        let mut res = SmallSetFunc::from_sorted_vec(self.universe.clone());
        res.values = self.values.clone();
        for (bitset, value) in rhs.values.iter() {
            *res.values.entry(*bitset).or_default() -= value;            
        }

        res
    }
}

impl fmt::Display for SmallSetFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("U={:?} ", self.universe))?;
        f.write_str("{")?;
        for (set, value) in self.entries_nonzero() {
            f.write_str(&format!("{set:?}: {value},"))?;
        }

        f.write_str("}")?;
        Ok(())
    }
}