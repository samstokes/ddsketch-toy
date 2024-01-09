use std::collections::BTreeMap;

#[cfg(test)]
use proptest::collection::vec;
#[cfg(test)]
use proptest::{prop_assert_eq, proptest};

pub struct Sketch {
    // alpha: f32, // TODO
    gamma: f32,
    log2_gamma_inv: f32,
    two_over_gamma_plus_1: f32,
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
        if let Some(count) = self.buckets.get_mut(&i) {
            *count += 1;
        } else {
            self.buckets.insert(i, 1);
        }
    }

    pub fn size(&self) -> usize {
        self.buckets.values().sum()
    }

    pub fn quantile(&self, q: f32) -> f32 {
        assert!(q >= 0.0 && q <= 1.0);
        assert!(!self.buckets.is_empty());
        // TODO rewrite this nicely using iters
        let (&i_0, &count_0) = self.buckets.first_key_value().unwrap();

        // TODO handle q > 0
        assert_eq!(0.0, q);

        self.two_over_gamma_plus_1 * self.gamma.powi(i_0 as i32)
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
        fn has_size(items in vec(0.0f32..1e8, 0..10)) {
            let mut s = Sketch::new(TEST_GAMMA);
            for &item in &items {
                s.insert(item);
            }
            // TODO generate variable arrays of values instead of 1
            prop_assert_eq!(items.len(), s.size());
        }
    }
}
