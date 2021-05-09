use rand::Rng;
use rand::seq::index::sample_weighted;
use std::mem;

fn validate_inputs(
    len: usize,
    deny: (usize, usize),
) {
    let (deny_a, deny_b) = deny;
    if len < 3 {
        panic!("not enough indices to pick from")
    }
    if deny_a == deny_b {
        panic!("denied indices must be distinct")
    }
    if deny_a >= len {
        panic!(
            "tuple {:?} is not fully contained in range {:?}",
            deny,
            0..len
        )
    }
    if deny_b >= len {
        panic!(
            "tuple {:?} is not fully contained in range {:?}",
            deny,
            0..len
        )
    }
}

pub trait RngExt: Rng {
    /// Select two distinct indices from 0..len. Return the indices as an ordered tuple.
    /// Never return the given `deny` tuple.
    /// Every possible candidate tuple is selected with uniform probability.
    fn random_distinct_index_tuple_ordered_except_good(
        &mut self,
        len: usize,
        deny: (usize, usize),
    ) -> (usize, usize) {
        validate_inputs(len, deny);
        let (deny_a, deny_b) = deny;
        let (mut a, mut b) = loop {
            let indices = rand::seq::index::sample(self, len, 2);
            let (a, b) = (indices.index(0), indices.index(1));
            if (a != deny_a && a != deny_b) || (b != deny_a && b != deny_b) {
                break (a, b);
            }
        };

        if a > b { mem::swap(&mut a, &mut b); }
        (a, b)
    }

    /// Select two distinct indices from 0..len. Return the indices as an ordered tuple.
    /// Never return the given `deny` tuple.
    /// Every possible candidate tuple is selected with uniform probability.
    fn random_distinct_index_tuple_ordered_except_fast(
        &mut self,
        len: usize,
        deny: (usize, usize),
    ) -> (usize, usize) {
        validate_inputs(len, deny);
        let (mut deny_a, mut deny_b) = deny;

        let mut a = self.gen_range(0..len);

        let mut b = if a == deny_a || a == deny_b {
            // sort ascending: deny_a, deny_b
            if deny_a > deny_b { mem::swap(&mut deny_a, &mut deny_b); }
            let ranges = vec![
                0..deny_a,
                deny_a + 1..deny_b,
                deny_b + 1..len,
            ];
            let idx = sample_weighted(self, 3, |i| {
                ranges[i].len() as f64
            }, 1).unwrap().index(0);
            self.gen_range(ranges[idx].clone())
        } else {
            let b = self.gen_range(0..len - 1);
            if a == b {
                len - 1
            } else {
                b
            }
        };

        if a > b { mem::swap(&mut a, &mut b); }
        (a, b)
    }
}

impl<R: Rng + ?Sized> RngExt for R {}

#[cfg(test)]
mod test {
    use std::hash::Hash;
    use std::collections::{HashMap, BTreeMap, BTreeSet};
    use rand_pcg::Pcg64;
    use rand::SeedableRng;
    use maplit::btreemap;
    use crate::rand_indices::RngExt;

    #[test]
    fn random_distinct_index_tuple_ordered_except_good_ok_cases() {
        let mut rng = Pcg64::seed_from_u64(1);
        struct TestCase {
            name: &'static str,
            input_len: usize,
            deny: (usize, usize),
            expected_dist: BTreeMap<(usize, usize), usize>,
        }
        let tests = vec![
            TestCase {
                name: "three elems, start&end denied",
                input_len: 3,
                deny: (0, 2),
                expected_dist: btreemap! {
                    (0,1) => 50,
                    (1,2) => 50,
                },
            },
            TestCase {
                name: "four elems, start&end denied",
                input_len: 4,
                deny: (0, 3),
                expected_dist: btreemap! {
                    (0,1) => 20,
                    (0,2) => 20,
                    (1,2) => 20,
                    (1,3) => 20,
                    (2,3) => 20,
                },
            },
            TestCase {
                name: "four elems, two non-adjacent denied",
                input_len: 4,
                deny: (0, 2),
                expected_dist: btreemap! {
                    (0,1) => 20,
                    (0,3) => 20,
                    (1,2) => 20,
                    (1,3) => 20,
                    (2,3) => 20,
                },
            },
            TestCase {
                name: "five elems, two adjacent denied",
                input_len: 5,
                deny: (1, 2),
                expected_dist: btreemap! {
                    (0,1) => 11,
                    (0,2) => 11,
                    (0,3) => 11,
                    (0,4) => 11,
                    (1,3) => 11,
                    (1,4) => 11,
                    (2,3) => 11,
                    (2,4) => 11,
                    (3,4) => 11,
                },
            },
        ];
        for i in 0..tests.len() {
            let test = &tests[i];
            let actual_dist = repeat_and_collect(|| {
                rng.random_distinct_index_tuple_ordered_except_good(test.input_len, test.deny)
            });
            assert_eq!(
                test.expected_dist.keys().collect::<BTreeSet<_>>(),
                actual_dist.keys().collect::<BTreeSet<_>>(),
                "test {} failed",
                test.name,
            );
        }
        for i in 0..tests.len() {
            let test = &tests[i];
            let actual_dist = repeat_and_collect(|| {
                rng.random_distinct_index_tuple_ordered_except_good(test.input_len, test.deny)
            });
            assert_eq!(test.expected_dist, actual_dist, "test {} failed", test.name);
        }
    }

    #[test]
    fn random_distinct_index_tuple_ordered_except_fast_ok_cases() {
        let mut rng = Pcg64::seed_from_u64(1);
        struct TestCase {
            name: &'static str,
            input_len: usize,
            deny: (usize, usize),
            expected_dist: BTreeMap<(usize, usize), usize>,
        }
        let tests = vec![
            TestCase {
                name: "three elems, start&end denied",
                input_len: 3,
                deny: (0, 2),
                expected_dist: btreemap! {
                    (0,1) => 50,
                    (1,2) => 50,
                },
            },
            TestCase {
                name: "four elems, start&end denied",
                input_len: 4,
                deny: (0, 3),
                expected_dist: btreemap! {
                    (0,1) => 20,
                    (0,2) => 20,
                    (1,2) => 20,
                    (1,3) => 20,
                    (2,3) => 20,
                },
            },
            TestCase {
                name: "four elems, two non-adjacent denied",
                input_len: 4,
                deny: (0, 2),
                expected_dist: btreemap! {
                    (0,1) => 20,
                    (0,3) => 20,
                    (1,2) => 20,
                    (1,3) => 20,
                    (2,3) => 20,
                },
            },
            TestCase {
                name: "five elems, two adjacent denied",
                input_len: 5,
                deny: (1, 2),
                expected_dist: btreemap! {
                    (0,1) => 11,
                    (0,2) => 11,
                    (0,3) => 11,
                    (0,4) => 11,
                    (1,3) => 11,
                    (1,4) => 11,
                    (2,3) => 11,
                    (2,4) => 11,
                    (3,4) => 11,
                },
            },
        ];
        for i in 0..tests.len() {
            let test = &tests[i];
            let actual_dist = repeat_and_collect(|| {
                rng.random_distinct_index_tuple_ordered_except_fast(test.input_len, test.deny)
            });
            assert_eq!(
                test.expected_dist.keys().collect::<BTreeSet<_>>(),
                actual_dist.keys().collect::<BTreeSet<_>>(),
                "test {} failed",
                test.name,
            );
        }
        for i in 0..tests.len() {
            let test = &tests[i];
            let actual_dist = repeat_and_collect(|| {
                rng.random_distinct_index_tuple_ordered_except_fast(test.input_len, test.deny)
            });
            assert_eq!(test.expected_dist, actual_dist, "test {} failed", test.name);
        }
    }

    pub fn repeat_and_collect<T, F>(mut function: F) -> BTreeMap<T, usize>
        where
            T: Eq + Hash + Ord,
            F: FnMut() -> T,
    {
        let mut counters = HashMap::new();
        for _ in 0..100_000 {
            let selected = function();
            counters
                .entry(selected)
                .and_modify(|count| {
                    *count += 1;
                })
                .or_insert(1);
        }
        counters
            .into_iter()
            .map(|(value, count)| (value, ((count as f32 / 1000.0).round()) as usize))
            .collect()
    }
}