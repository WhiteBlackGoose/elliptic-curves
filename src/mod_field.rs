use std::ops;

use rand::Rng;

use crate::groups::{
    self, AbelianGroup, CommutativeOp, Field, Identity, Inverse, InverseNonZero, Natural,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModFieldCfg<I> {
    rem: I,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModField<I: Natural> {
    val: I,
}

fn gcd<S>(a: S, b: S) -> S
where
    S: Natural,
{
    if a == b {
        return a;
    }
    if a == S::zero() {
        return b;
    }
    if b == S::zero() {
        return a;
    }
    if a > b {
        gcd(a % b, b)
    } else {
        gcd(a, b % a)
    }
}

impl<I: Natural> CommutativeOp<groups::ops::Add, ModFieldCfg<I>> for ModField<I> {
    fn op(a: Self, b: Self, c: &ModFieldCfg<I>) -> Self {
        todo!()
    }
}

impl<I: Natural> Inverse<groups::ops::Add, ModFieldCfg<I>> for ModField<I> {
    fn inv(self, cfg: &ModFieldCfg<I>) -> Self {
        Self {
            val: cfg.rem - self.val,
        }
    }
}

impl<I: Natural> Identity<groups::ops::Add, ModFieldCfg<I>> for ModField<I> {
    fn identity(c: &ModFieldCfg<I>) -> Self {
        Self { val: I::zero() }
    }
}

impl<I: Natural> AbelianGroup<groups::ops::Add, ModFieldCfg<I>> for ModField<I> {
    fn zero(_cfg: &ModFieldCfg<I>) -> Self {
        Self { val: I::zero() }
    }
}

impl<I: Natural> CommutativeOp<groups::ops::Mul, ModFieldCfg<I>> for ModField<I> {
    fn op(a: Self, b: Self, c: &ModFieldCfg<I>) -> Self {
        todo!()
    }
}

impl<I: Natural> InverseNonZero<groups::ops::Mul, ModFieldCfg<I>> for ModField<I> {
    fn inv(self, c: &ModFieldCfg<I>) -> Option<Self> {
        assert!(gcd(c.rem, self.nat()) == I::one(), "can't mul invert");
        // Little Fermat's theorem
        self.pow(c.rem - I::two())
    }
}

impl<I: Natural> Identity<groups::ops::Mul, ModFieldCfg<I>> for ModField<I> {
    fn identity(_c: &ModFieldCfg<I>) -> Self {
        Self { val: I::one() }
    }
}

impl<I: Natural> Field<ModFieldCfg<I>> for ModField<I> {}

impl<F: Natural> ModField<F> {
    pub fn nat(self) -> F {
        self.v
    }

    pub fn invert(self) -> Self {}

    pub fn pow(self, p: S) -> Self {
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
        let l = r.next_S();
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

impl<const P: S> std::fmt::Display for ModField<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.v.fmt(f)
    }
}

impl<const P: S> std::fmt::Debug for ModField<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.v.fmt(f)
    }
}

impl<const P: S> ModField<P> {
    pub const fn new(p: S) -> Self {
        Self { v: p % P }
    }
}

impl<const P: S> ops::Add for ModField<P> {
    type Output = ModField<P>;

    fn add(self, rhs: Self) -> Self::Output {
        let v1 = self.v as i128;
        let v2 = rhs.v as i128;
        Self {
            v: ((v1 + v2) % (P as i128)) as S,
        }
    }
}

impl<const P: S> ops::Mul for ModField<P> {
    type Output = ModField<P>;

    fn mul(self, rhs: Self) -> Self::Output {
        let v1 = self.v as i128;
        let v2 = rhs.v as i128;
        Self {
            v: ((v1 * v2) % (P as i128)) as S,
        }
    }
}

impl<const P: S> ops::Sub for ModField<P> {
    type Output = ModField<P>;

    fn sub(self, rhs: Self) -> Self::Output {
        let v1 = self.v as i128;
        let v2 = rhs.v as i128;
        Self {
            v: ((v1 - v2).rem_euclid(P as i128)) as S,
        }
    }
}

impl<const P: S> ops::Neg for ModField<P> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { v: P - self.v }
    }
}

impl<const P: S> ops::Div for ModField<P> {
    type Output = ModField<P>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: Self) -> Self::Output {
        self * rhs.invert()
    }
}

#[cfg(test)]
mod tests {
    use rand::{RngCore, SeedableRng};

    use crate::mod_field::{gcd, ModField};
    type F = ModField<19>;

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
