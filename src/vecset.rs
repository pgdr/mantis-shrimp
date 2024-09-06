pub fn union(A: &[u32], B: &[u32]) -> Vec<u32> {
    let mut res = Vec::default();
    let mut ia = 0;
    let mut ib = 0;
    while ia < A.len() && ib < B.len() {
        let (a, b) = (A[ia], B[ib]);
        if a == b {
            res.push(a);
            ia += 1;
            ib += 1;
        } else if a < b {
            res.push(a);
            ia += 1;
        } else {
            // a > b
            res.push(b);
            ib += 1;
        }
    }

    // Handle remainders:
    // One of these slices will be empty and we want
    // to append the non-empty one to the end of the result.
    res.extend(&A[ia..]);
    res.extend(&B[ib..]);

    res
}

pub fn intersection(A: &[u32], B: &[u32]) -> Vec<u32> {
    let mut res = Vec::default();
    let mut ia = 0;
    let mut ib = 0;
    while ia < A.len() && ib < B.len() {
        let (a, b) = (A[ia], B[ib]);
        if a == b {
            res.push(a);
            ia += 1;
            ib += 1;
        } else if a < b {
            ia += 1;
        } else {
            // a > b
            ib += 1;
        }
    }

    // Handle remainders:
    // Cannot be in intersection.

    res
}

pub fn difference(A: &[u32], B: &[u32]) -> Vec<u32> {
    let mut res = Vec::default();
    let mut ia = 0;
    let mut ib = 0;
    while ia < A.len() && ib < B.len() {
        let (a, b) = (A[ia], B[ib]);
        if a == b {
            // We know that a is not in the result
            ia += 1;
            ib += 1;
        } else if a < b {
            // We can be sure that a is not in B
            res.push(a);
            ia += 1;
        } else {
            // a > b
            ib += 1;
        }
    }

    // Handle remainders:
    // - If B has leftovers then A is done and we are finished
    // - If A has leftovers then B is done, so we need to append the remainder of A
    // In either case we can append the remainder of A.
    res.extend(&A[ia..]);

    res
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use rand::prelude::*;

    fn rand_set(rng: &mut ThreadRng) -> (Vec<u32>, BTreeSet<u32>) {
        let mut res = Vec::default();
        for _ in 0..50 {
            res.push(rng.gen_range(0..100));
        }
        res.sort_unstable();
        res.dedup();

        let set = res.iter().cloned().collect();
        (res, set)
    }

    #[test]
    fn test_union() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let (A, A_set) = rand_set(&mut rng);
            let (B, B_set) = rand_set(&mut rng);

            let res = union(&A, &B);
            let res_set: Vec<u32> = A_set.union(&B_set).cloned().collect();
            assert_eq!(res, res_set);

            assert_eq!(res, union(&B, &A));
            assert_eq!(union(&A, &vec![]), A);
            assert_eq!(union(&B, &vec![]), B);
        }
    }

    #[test]
    fn test_intersection() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let (A, A_set) = rand_set(&mut rng);
            let (B, B_set) = rand_set(&mut rng);

            let res = intersection(&A, &B);
            let res_set: Vec<u32> = A_set.intersection(&B_set).cloned().collect();
            assert_eq!(res, res_set);
            assert_eq!(res, intersection(&B, &A));
            assert_eq!(intersection(&A, &vec![]), vec![]);
            assert_eq!(intersection(&B, &vec![]), vec![]);
        }
    }

    #[test]
    fn test_difference() {
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let (A, A_set) = rand_set(&mut rng);
            let (B, B_set) = rand_set(&mut rng);

            let res = difference(&A, &B);
            let res_set: Vec<u32> = A_set.difference(&B_set).cloned().collect();
            assert_eq!(res, res_set);
            assert_eq!(difference(&A, &vec![]), A);
            assert_eq!(difference(&vec![], &A), vec![]);
            assert_eq!(difference(&A, &A), vec![]);
        }
    }
}
