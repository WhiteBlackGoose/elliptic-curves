use std::{
    fmt::{Debug, Display},
    ops,
};

use rand::Rng;

use crate::{
    algebra::{
        self, AbelianGroup, CommutativeMonoid, CommutativeOp, Configurable, DiscreteRoot, Field,
        Identity, Inverse, InverseNonZero,
    },
    base_traits::{FromRandom, Natural},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModFieldCfg<I> {
    pub rem: I,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModField<I: Natural> {
    val: I,
}

impl<I: Natural> Configurable for ModField<I> {
    type Cfg = ModFieldCfg<I>;
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

impl<I: Natural> CommutativeOp<algebra::ops::Add> for ModField<I> {
    fn op(a: Self, b: Self, c: &ModFieldCfg<I>) -> Self {
        todo!()
    }
}

impl<I: Natural> Inverse<algebra::ops::Add> for ModField<I> {
    fn inv(self, cfg: &ModFieldCfg<I>) -> Self {
        Self {
            val: cfg.rem - self.val,
        }
    }
}

impl<I: Natural> Identity<algebra::ops::Add> for ModField<I> {
    fn identity(c: &ModFieldCfg<I>) -> Self {
        Self { val: I::zero() }
    }
}

impl<I: Natural> CommutativeOp<algebra::ops::Mul> for ModField<I> {
    fn op(a: Self, b: Self, c: &ModFieldCfg<I>) -> Self {
        todo!()
    }
}
impl<I: Natural> Identity<algebra::ops::Mul> for ModField<I> {
    fn identity(_c: &ModFieldCfg<I>) -> Self {
        Self { val: I::one() }
    }
}

impl<I: Natural> CommutativeMonoid<algebra::ops::Add> for ModField<I> {}
impl<I: Natural> CommutativeMonoid<algebra::ops::Mul> for ModField<I> {}
impl<I: Natural> AbelianGroup<algebra::ops::Add> for ModField<I> {}

impl<I: Natural> InverseNonZero<algebra::ops::Mul> for ModField<I> {
    fn inv(self, c: &ModFieldCfg<I>) -> Option<Self> {
        if gcd(c.rem, self.nat()) != I::one() {
            return None;
        }
        // Little Fermat's theorem
        Some(CommutativeMonoid::<algebra::ops::Mul>::exp(
            self,
            c.rem - I::two(),
            c,
        ))
    }
}

impl<I: Natural> Field for ModField<I> {}

impl<I: Natural> DiscreteRoot<algebra::ops::Mul> for ModField<I> {
    fn sqrt(self, c: &ModFieldCfg<I>) -> Option<Self> {
        if self.pow((c.rem - I::one()) / I::two(), c) != Self::one(c) {
            return None;
        }
        let three = I::two() + I::one();
        let four = I::two() + I::two();
        if c.rem % four == three {
            Some(self.pow((c.rem + I::one()) / four, c))
        } else {
            todo!();
        }
    }
}

impl<I: Natural> ModField<I> {
    pub fn new(p: I, cfg: &ModFieldCfg<I>) -> Self {
        Self { val: p % cfg.rem }
    }
    pub fn nat(self) -> I {
        self.val
    }
}
impl<I: Natural + FromRandom> ModField<I> {
    pub fn random<R: Rng>(r: &mut R, cfg: &ModFieldCfg<I>) -> Self {
        Self::new(I::random(r), cfg)
    }

    pub fn random_nonzero<R: Rng>(r: &mut R, cfg: &ModFieldCfg<I>) -> Self {
        loop {
            let r = Self::random(r, cfg);
            if r != Self::zero(cfg) {
                return r;
            }
        }
    }
}

impl<I: Natural + Display> std::fmt::Display for ModField<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.val.fmt(f)
    }
}

impl<I: Natural + Debug> std::fmt::Debug for ModField<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.val.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use rand::{RngCore, SeedableRng};

    use crate::{
        algebra::Field,
        mod_field::{gcd, ModField},
    };

    use super::ModFieldCfg;

    type F = ModField<u64>;

    fn cfg() -> ModFieldCfg<u64> {
        ModFieldCfg { rem: 19 }
    }

    fn f(a: u64) -> F {
        F::new(a, &cfg())
    }

    #[test]
    fn simple() {
        assert_eq!(f(27), f(8));
    }

    #[test]
    fn add() {
        assert_eq!(F::add(f(7), f(13), &cfg()), f(1));
    }

    #[test]
    fn mul() {
        assert_eq!(F::mul(f(7), f(13), &cfg()), f(15));
    }

    #[test]
    fn sub() {
        assert_eq!(F::sub(f(7), f(13), &cfg()), f(13));
    }

    #[test]
    fn div() {
        assert_eq!(F::div(f(11), f(5), &cfg()), f(6));
    }

    #[test]
    fn inv() {
        assert_eq!(f(11).reciprocal(&cfg()), Some(f(7)));
    }

    #[test]
    fn neg() {
        assert_eq!(f(11).neg(&cfg()), f(8));
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
            let a = F::random(&mut gen, &cfg());
            let b = F::random_nonzero(&mut gen, &cfg());
            assert_eq!(
                F::mul(F::div(a, b, &cfg()), b, &cfg()),
                a,
                "a: {}, b: {}, a/b: {}",
                a,
                b,
                F::div(a, b, &cfg())
            );
        }
    }

    #[test]
    fn inv_circ() {
        let mut gen = rand_chacha::ChaCha8Rng::from_seed([1u8; 32]);
        for _ in 0..100 {
            let a = F::random_nonzero(&mut gen, &cfg());
            assert_eq!(
                F::mul(a.reciprocal(&cfg()).unwrap(), a, &cfg()),
                F::one(&cfg()),
                "a: {}, a^-1: {}",
                a,
                a.reciprocal(&cfg()).unwrap()
            );
        }
    }

    #[test]
    fn sub_circ() {
        let mut gen = rand_chacha::ChaCha8Rng::from_seed([1u8; 32]);
        for _ in 0..100 {
            let a = F::random(&mut gen, &cfg());
            let b = F::random(&mut gen, &cfg());
            assert_eq!(
                F::add(F::sub(a, b, &cfg()), b, &cfg()),
                a,
                "a: {}, b: {}",
                a,
                b
            );
        }
    }
}
