use std::collections::BTreeMap;

#[cfg(test)]
use proptest::collection::vec;
#[cfg(test)]
use proptest::{prop_assert, prop_assert_eq, proptest};

pub struct Sketch {
    // alpha: f32, // TODO
    gamma: f32,
    log2_gamma_inv: f32,
    two_over_gamma_plus_1: f32,
    num_values: usize,
    // TODO i16 probably insufficient range for f32
    buckets: BTreeMap<i16, usize>,
}

impl Sketch {
    pub fn new(alpha: f32) -> Self {
        assert!(alpha > 0.0 && alpha < 1.0);
        let gamma = (1.0 + alpha) / (1.0 - alpha);
        let buckets = Default::default();
        Self {
            gamma,
            log2_gamma_inv: 1.0 / gamma.log2(),
            two_over_gamma_plus_1: 2.0 / (gamma + 1.0),
            num_values: 0,
            buckets,
        }
    }

    pub fn insert(&mut self, item: f32) {
        // want i such that item between [gamma^(i-1) .. gamma^i]
        // log_gamma(item) between i-1 .. i
        // ceil(log_gamma(item)) = i
        // i = ceil(log2(item) / log2(gamma))
        // this may behave strangely for item within EPSILON of 0, TODO check and maybe handle specially
        // (in particular 0 -> INT_MIN and -0 -> INT_MAX)
        let i = (item.log2() * self.log2_gamma_inv).ceil() as i16;
        println!("item {} -> bucket {}", item, i);
        if let Some(count) = self.buckets.get_mut(&i) {
            *count += 1;
        } else {
            self.buckets.insert(i, 1);
        }
        self.num_values += 1;
    }

    pub fn size(&self) -> usize {
        self.num_values
    }

    #[cfg(test)]
    fn bucket_sum(&self) -> usize {
        self.buckets.values().sum()
    }

    pub fn quantile(&self, q: f32) -> f32 {
        assert!(q >= 0.0 && q <= 1.0);
        assert!(!self.buckets.is_empty());

        let mut buckets = self.buckets.iter();

        // TODO rewrite this nicely using a reduce or something
        let (&i_0, &count_0) = buckets.next().unwrap();
        println!("i_0: {}, count_0: {}", i_0, count_0);

        let target_rank = q * (self.num_values - 1) as f32;
        println!("q: {}, target_rank: {}", q, target_rank);
        let (mut i, mut count) = (i_0, count_0);
        while count as f32 <= target_rank {
            let (&i_next, &count_next) = buckets.next().expect("ran out of buckets early");
            (i, count) = (i_next, count + count_next);
            println!("i: {}, count: {}", i, count);
        }

        self.two_over_gamma_plus_1 * self.gamma.powi(i as i32)
    }
}

#[cfg(test)]
const TEST_GAMMA: f32 = 0.05;

#[cfg(test)]
mod tests {
    use super::*;

    fn close(expected: f32, actual: f32) -> bool {
        const LOWER: f32 = 1.0 - TEST_GAMMA;
        const UPPER: f32 = 1.0 + TEST_GAMMA;
        expected * LOWER <= actual && actual <= expected * UPPER
    }

    #[test]
    fn should_create() {
        Sketch::new(TEST_GAMMA);
    }

    #[test]
    fn should_insert_positive_numbers() {
        let mut s = Sketch::new(TEST_GAMMA);
        s.insert(0.1);
        s.insert(0.2);
        s.insert(1000.0);
        s.insert(1e8);

        assert_eq!(4, s.size());

        assert!(close(0.1, s.quantile(0.0)));
        assert!(close(1e8, s.quantile(1.0)));
    }

    #[test]
    fn should_insert_zero() {
        let mut s = Sketch::new(TEST_GAMMA);
        s.insert(0.0);
        s.insert(0.1);
        s.insert(1.0);

        assert_eq!(3, s.size());
        assert_eq!(0.0, s.quantile(0.0));
    }

    proptest! {
        // TODO would like to test with larger arrays but it takes minutes to execute
        #[test]
        fn non_negative_items(items in vec(0.0_f32..1e8, 1..10)) {
            let mut s = Sketch::new(TEST_GAMMA);
            for &item in &items {
                s.insert(item);
            }
            prop_assert_eq!(items.len(), s.size());
            prop_assert_eq!(s.bucket_sum(), s.size());

            let mut sorted = items.clone();
            sorted.sort_unstable_by(f32::total_cmp);

            for pct in (0..=100).step_by(10) {
                let q = pct as f32 / 100.0;
                // value such that pct% of values are <= it

                let target_index = (items.len() - 1) as f32 * q;
                println!("n: {}, pct: {}, q: {}, targrank: {}", items.len(), pct, q, target_index);
                let target_index_floor = target_index.floor() as usize;
                println!("floor: {}", target_index_floor);
                let value = sorted[target_index_floor];

                let estimate = s.quantile(q);
                println!("value: {}, estimate: {}", value, estimate);
                prop_assert!(close(value, estimate));
            }
        }
    }
}
