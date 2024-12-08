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
            let r = self.exp(n, cfg);
            if m % I::two() == I::zero() {
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
