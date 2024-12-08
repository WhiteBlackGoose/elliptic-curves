use crate::base_traits::Natural;

pub trait Configurable: Sized + Copy {
    type Cfg;
}

pub trait CommutativeOp<Op>: Configurable {
    fn op(a: Self, b: Self, c: &Self::Cfg) -> Self;

    fn exp<I: Natural>(self, n: I, cfg: &Self::Cfg) -> Self {
        if n == I::zero() {
            panic!("Identity element for power 0 is not defined, use Monoid::exp");
        } else if n == I::one() {
            self
        } else {
            let m = n / I::two();
            let r = self.exp(m, cfg);
            if n % I::two() == I::zero() {
                CommutativeOp::op(r, r, cfg)
            } else {
                CommutativeOp::op(r, CommutativeOp::op(r, self, cfg), cfg)
            }
        }
    }
}

pub trait Identity<Op>: Configurable {
    fn identity(c: &Self::Cfg) -> Self;
}

pub trait Inverse<Op>: Configurable {
    fn inv(self, c: &Self::Cfg) -> Self;
}

pub trait InverseNonZero<Op>: Configurable {
    fn inv(self, c: &Self::Cfg) -> Option<Self>;
}

pub trait CommutativeMonoid<Op>: CommutativeOp<Op> + Identity<Op> {
    fn exp<I: Natural>(self, n: I, cfg: &Self::Cfg) -> Self {
        if n == I::zero() {
            Identity::identity(cfg)
        } else {
            CommutativeOp::exp(self, n, cfg)
        }
    }
}

pub trait AbelianGroup<Op>: CommutativeMonoid<Op> + Inverse<Op> {}

pub trait DiscreteRoot<Op>: CommutativeOp<Op> {
    fn sqrt(self, cfg: &Self::Cfg) -> Option<Self>;
}

pub trait InitialPoint<P> {
    fn g(&self) -> P;
}

pub mod ops {
    pub struct Add;
    pub struct Mul;
}

pub trait Field:
    Sized + AbelianGroup<ops::Add> + CommutativeMonoid<ops::Mul> + InverseNonZero<ops::Mul> + Eq
{
    fn add(a: Self, b: Self, cfg: &Self::Cfg) -> Self {
        CommutativeOp::<ops::Add>::op(a, b, cfg)
    }
    fn sub(a: Self, b: Self, cfg: &Self::Cfg) -> Self {
        CommutativeOp::<ops::Add>::op(a, Inverse::<ops::Add>::inv(b, cfg), cfg)
    }
    fn mul(a: Self, b: Self, cfg: &Self::Cfg) -> Self {
        CommutativeOp::<ops::Mul>::op(a, b, cfg)
    }
    fn div(a: Self, b: Self, cfg: &Self::Cfg) -> Self {
        CommutativeOp::<ops::Mul>::op(a, InverseNonZero::<ops::Mul>::inv(b, cfg).unwrap(), cfg)
    }
    fn zero(cfg: &Self::Cfg) -> Self {
        Identity::<ops::Add>::identity(cfg)
    }
    fn one(cfg: &Self::Cfg) -> Self {
        Identity::<ops::Mul>::identity(cfg)
    }
    fn two(cfg: &Self::Cfg) -> Self {
        let one = Self::one(cfg);
        Self::add(one, one, cfg)
    }
    fn three(cfg: &Self::Cfg) -> Self {
        Self::add(Self::two(cfg), Self::one(cfg), cfg)
    }
    fn four(cfg: &Self::Cfg) -> Self {
        let two = Self::two(cfg);
        Self::add(two, two, cfg)
    }
    fn pow<N: Natural>(self, n: N, cfg: &Self::Cfg) -> Self {
        CommutativeMonoid::<ops::Mul>::exp(self, n, cfg)
    }
    fn reciprocal(self, cfg: &Self::Cfg) -> Option<Self> {
        InverseNonZero::inv(self, cfg)
    }
    fn neg(self, cfg: &Self::Cfg) -> Self {
        Inverse::inv(self, cfg)
    }
    fn sqr(self, cfg: &Self::Cfg) -> Self {
        Self::mul(self, self, cfg)
    }
    fn cube(self, cfg: &Self::Cfg) -> Self {
        Self::mul(Self::sqr(self, cfg), self, cfg)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        algebra::{CommutativeMonoid, Identity},
        mod_field::ModFieldCfg,
    };

    use super::{ops, CommutativeOp, Configurable};

    #[derive(Clone, Copy)]
    struct Q {
        val: u32,
    }
    impl Configurable for Q {
        type Cfg = ();
    }
    impl CommutativeOp<ops::Add> for Q {
        fn op(a: Self, b: Self, _c: &Self::Cfg) -> Self {
            Q { val: a.val + b.val }
        }
    }
    #[test]
    fn exp1() {
        let q = Q { val: 7 };
        assert_eq!(CommutativeOp::exp(q, 9u64, &()).val, 63);
    }
    #[test]
    fn exp2() {
        let q = Q { val: 7 };
        assert_eq!(CommutativeOp::exp(q, 6u64, &()).val, 42);
    }
    #[test]
    fn exp3() {
        let q = Q { val: 7 };
        assert_eq!(CommutativeOp::exp(q, 1u64, &()).val, 7);
    }
    #[test]
    #[should_panic]
    fn exp4() {
        let q = Q { val: 7 };
        CommutativeOp::exp(q, 0u64, &());
    }
    #[test]
    fn exp5() {
        impl Identity<ops::Add> for Q {
            fn identity(_c: &Self::Cfg) -> Self {
                Self { val: 1234 }
            }
        }
        impl CommutativeMonoid<ops::Add> for Q {}
        let q = Q { val: 7 };
        assert_eq!(CommutativeMonoid::exp(q, 0u64, &()).val, 1234);
    }
}
