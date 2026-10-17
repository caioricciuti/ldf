//! Self-contained deterministic PRNG: xoshiro256** seeded through
//! splitmix64. No external crate so the stream can never change under a
//! dependency bump; the algorithms are frozen and documented here.
//!
//! Reference: Blackman and Vigna, "Scrambled Linear Pseudorandom Number
//! Generators" (xoshiro256**), and Steele, Lea, Flood (splitmix64).

#[derive(Debug, Clone)]
pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        let mut x = seed;
        let mut next = || {
            x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = x;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        };
        Rng {
            s: [next(), next(), next(), next()],
        }
    }

    /// Independent sub-stream for record `index` of dataset `seed`, so any
    /// record can be regenerated without replaying the ones before it.
    pub fn for_record(seed: u64, index: u64) -> Rng {
        Rng::new(seed ^ index.wrapping_mul(0x9E37_79B9_7F4A_7C15).rotate_left(17))
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// Uniform in `0..n` (Lemire's nearly divisionless method).
    pub fn below(&mut self, n: u64) -> u64 {
        debug_assert!(n > 0);
        let mut m = (self.next_u64() as u128) * (n as u128);
        let mut l = m as u64;
        if l < n {
            let t = n.wrapping_neg() % n;
            while l < t {
                m = (self.next_u64() as u128) * (n as u128);
                l = m as u64;
            }
        }
        (m >> 64) as u64
    }

    pub fn range_i64(&mut self, lo: i64, hi_inclusive: i64) -> i64 {
        lo + self.below((hi_inclusive - lo + 1) as u64) as i64
    }

    /// Uniform in `[0, 1)` with 53 bits of randomness.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    pub fn chance(&mut self, p: f64) -> bool {
        self.unit() < p
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len() as u64) as usize]
    }

    /// Skewed pick: index `i` has weight `weights[i]`.
    pub fn pick_weighted<'a, T>(&mut self, items: &'a [T], weights: &[u32]) -> &'a T {
        let total: u64 = weights.iter().map(|&w| w as u64).sum();
        let mut r = self.below(total);
        for (item, &w) in items.iter().zip(weights) {
            if r < w as u64 {
                return item;
            }
            r -= w as u64;
        }
        &items[items.len() - 1]
    }

    pub fn alnum(&mut self, len: usize) -> String {
        const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        (0..len).map(|_| *self.pick(CHARS) as char).collect()
    }

    pub fn hex(&mut self, len: usize) -> String {
        const CHARS: &[u8] = b"0123456789abcdef";
        (0..len).map(|_| *self.pick(CHARS) as char).collect()
    }

    /// Float with `decimals` fractional digits in `[lo, hi)`, built from an
    /// integer so the value is exactly the printed decimal's nearest double.
    pub fn decimal(&mut self, lo: f64, hi: f64, decimals: u32) -> f64 {
        let scale = 10f64.powi(decimals as i32);
        let n = self.below(((hi - lo) * scale) as u64);
        let text = format!("{:.*}", decimals as usize, lo + n as f64 / scale);
        text.parse().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_and_reference_values() {
        // Frozen outputs: if these change, every fixture changes.
        let mut r = Rng::new(42);
        let first: Vec<u64> = (0..3).map(|_| r.next_u64()).collect();
        let mut r2 = Rng::new(42);
        let again: Vec<u64> = (0..3).map(|_| r2.next_u64()).collect();
        assert_eq!(first, again);
        assert_eq!(
            first,
            [
                1_546_998_764_402_558_742,
                6_990_951_692_964_543_102,
                12_544_586_762_248_559_009
            ]
        );
    }

    #[test]
    fn below_is_in_range_and_unit_in_bounds() {
        let mut r = Rng::new(1);
        for _ in 0..10_000 {
            assert!(r.below(7) < 7);
            let u = r.unit();
            assert!((0.0..1.0).contains(&u));
        }
    }
}
