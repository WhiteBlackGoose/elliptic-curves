use std::ops;

use rand::Rng;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Field<const P: u64> {
    v: u64,
}

fn gcd(a: u64, b: u64) -> u64 {
    if a == b {
        return a;
    }
    if a == 0 {
        return b;
    }
    if b == 0 {
        return a;
    }
    if a > b {
        gcd(a % b, b)
    } else {
        gcd(a, b % a)
    }
}

// returns: d, x, y
fn extended_gcd(a: u64, b: u64) -> (u64, i64, i64) {
    let (mut old_r, mut r) = (a as i64, b as i64);
    let (mut old_s, mut s) = (1, 0);
    let (mut old_t, mut t) = (0, 1);

    while r != 0 {
        let quotient = old_r / r;
        (old_r, r) = (r, old_r - quotient * r);
        (old_s, s) = (s, old_s - quotient * s);
        (old_t, t) = (t, old_t - quotient * t);
    }

    (old_r as u64, old_s, old_t) // gcd, Bezout coefficients
}

impl<const P: u64> Field<P> {
    pub fn nat(self) -> u64 {
        self.v
    }

    pub const fn zero() -> Self {
        Self::new(0)
    }

    pub const fn one() -> Self {
        Self::new(1)
    }

    pub fn invert(self) -> Self {
        assert!(gcd(P, self.v) == 1, "a: {}", self);
        // let (x, _y, _d) = extended_gcd(self.v as i64, P as i64);
        // let (_d, x, y) = extended_gcd(self.v, P);
        // Self::new(x as u64)
        self.pow(P - 2)
    }

    pub fn pow(self, p: u64) -> Self {
        if p == 0 {
            Self::new(1)
        } else if p == 1 {
            self
        } else {
            let m = self.pow(p / 2);
            let r = self.pow(p % 2);
            m * m * r
        }
    }

    pub fn random<R: Rng>(r: &mut R) -> Self {
        let l = r.next_u64();
        Self::new(l)
    }

    pub fn random_nonzero<R: Rng>(r: &mut R) -> Self {
        let res = Self::random(r);
        if res == Self::zero() {
            res + Self::one()
        } else {
            res
        }
    }

    pub fn sqrt(self) -> Option<Self> {
        if self.pow((P - 1) / 2) != Self::one() {
            return None;
        }
        if P % 4 == 3 {
            Some(self.pow((P + 1) / 4))
        } else {
            todo!();
        }
    }
}

impl<const P: u64> std::fmt::Display for Field<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.v.fmt(f)
    }
}

impl<const P: u64> std::fmt::Debug for Field<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.v.fmt(f)
    }
}

impl<const P: u64> Field<P> {
    pub const fn new(p: u64) -> Self {
        Self { v: p % P }
    }
}

impl<const P: u64> ops::Add for Field<P> {
    type Output = Field<P>;

    fn add(self, rhs: Self) -> Self::Output {
        let v1 = self.v as i128;
        let v2 = rhs.v as i128;
        Self {
            v: ((v1 + v2) % (P as i128)) as u64,
        }
    }
}

impl<const P: u64> ops::Mul for Field<P> {
    type Output = Field<P>;

    fn mul(self, rhs: Self) -> Self::Output {
        let v1 = self.v as i128;
        let v2 = rhs.v as i128;
        Self {
            v: ((v1 * v2) % (P as i128)) as u64,
        }
    }
}

impl<const P: u64> ops::Sub for Field<P> {
    type Output = Field<P>;

    fn sub(self, rhs: Self) -> Self::Output {
        let v1 = self.v as i128;
        let v2 = rhs.v as i128;
        Self {
            v: ((v1 - v2).rem_euclid(P as i128)) as u64,
        }
    }
}

impl<const P: u64> ops::Neg for Field<P> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { v: P - self.v }
    }
}

impl<const P: u64> ops::Div for Field<P> {
    type Output = Field<P>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: Self) -> Self::Output {
        self * rhs.invert()
    }
}

#[cfg(test)]
mod tests {
    use rand::{RngCore, SeedableRng};

    use crate::field::{gcd, Field};
    type F = Field<19>;

    #[test]
    fn simple() {
        assert_eq!(F::new(27), F::new(8));
    }

    #[test]
    fn add() {
        assert_eq!(F::new(7) + F::new(13), F::new(1));
    }

    #[test]
    fn mul() {
        assert_eq!(F::new(7) * F::new(13), F::new(15));
    }

    #[test]
    fn sub() {
        assert_eq!(F::new(7) - F::new(13), F::new(13));
    }

    #[test]
    fn div() {
        assert_eq!(F::new(11) / F::new(5), F::new(6));
    }

    #[test]
    fn inv() {
        assert_eq!(F::new(11).invert(), F::new(7));
    }

    #[test]
    fn neg() {
        assert_eq!(-F::new(11), F::new(8));
    }

    #[test]
    fn gcd1() {
        assert_eq!(gcd(11, 1), 1);
    }

    #[test]
    fn gcd2() {
        assert_eq!(gcd(12, 10), 2);
    }

    #[test]
    fn gcd3() {
        assert_eq!(gcd(1224832904, 1), 1);
    }

    #[test]
    fn gcd4() {
        assert_eq!(gcd(123, 66), 3);
    }

    #[test]
    fn div_circ() {
        let mut gen = rand_chacha::ChaCha8Rng::from_seed([1u8; 32]);
        for _ in 0..100 {
            let a = F::random(&mut gen);
            let b = F::random_nonzero(&mut gen);
            assert_eq!((a / b) * b, a, "a: {}, b: {}, a/b: {}", a, b, a / b);
        }
    }

    #[test]
    fn inv_circ() {
        let mut gen = rand_chacha::ChaCha8Rng::from_seed([1u8; 32]);
        for _ in 0..100 {
            let a = F::random_nonzero(&mut gen);
            assert_eq!(a.invert() * a, F::new(1), "a: {}, a^-1: {}", a, a.invert());
        }
    }

    #[test]
    fn sub_circ() {
        let mut gen = rand_chacha::ChaCha8Rng::from_seed([1u8; 32]);
        for _ in 0..100 {
            let a = F::random(&mut gen);
            let b = F::random(&mut gen);
            assert_eq!(a - b + b, a, "a: {}, b: {}", a, b);
        }
    }
}
