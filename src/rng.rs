//! A tiny, fast, seedable random number generator (xoshiro256++).
//!
//! The same seed always gives the same tree, on every machine.

use std::time::{SystemTime, UNIX_EPOCH};

pub struct Rng {
    s: [u64; 4],
}

fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        let mut st = seed;
        Rng {
            s: [
                splitmix(&mut st),
                splitmix(&mut st),
                splitmix(&mut st),
                splitmix(&mut st),
            ],
        }
    }

    fn next_u64(&mut self) -> u64 {
        let s = &mut self.s;
        let result = s[0].wrapping_add(s[3]).rotate_left(23).wrapping_add(s[0]);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }

    /// uniform in [0, 1)
    pub fn f(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }

    pub fn uniform(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.f()
    }

    /// integer in lo..=hi
    pub fn range(&mut self, lo: i64, hi: i64) -> i64 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u64() % ((hi - lo + 1) as u64)) as i64
    }

    pub fn below(&mut self, n: usize) -> usize {
        if n <= 1 {
            0
        } else {
            (self.next_u64() % n as u64) as usize
        }
    }

    pub fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.below(items.len())]
    }

    /// normal distribution (Box-Muller)
    pub fn gauss(&mut self, mu: f64, sigma: f64) -> f64 {
        let u1 = 1.0 - self.f(); // (0, 1]
        let u2 = self.f();
        mu + sigma * (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

/// a small random seed (1..=999999) taken from the clock, so printed seeds are easy to type
pub fn fresh_seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    let local = 0u8;
    let mut st = nanos ^ (&local as *const u8 as u64) ^ (std::process::id() as u64) << 32;
    splitmix(&mut st) % 999_999 + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_numbers() {
        let (mut a, mut b) = (Rng::new(12), Rng::new(12));
        for _ in 0..100 {
            assert_eq!(a.f().to_bits(), b.f().to_bits());
        }
    }

    #[test]
    fn ranges_hold() {
        let mut r = Rng::new(1);
        for _ in 0..10_000 {
            let v = r.uniform(-1.0, 2.0);
            assert!((-1.0..2.0).contains(&v));
            let k = r.range(3, 7);
            assert!((3..=7).contains(&k));
            assert!(r.below(5) < 5);
        }
    }

    #[test]
    fn seed_is_small() {
        let s = fresh_seed();
        assert!((1..=999_999).contains(&s));
    }
}
