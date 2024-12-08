use crate::base_traits::Natural;

pub trait CommutativeOp<Op, C>: Sized {
    fn op(a: Self, b: Self, c: &C) -> Self;
}

pub trait Identity<Op, C>: Sized {
    fn identity(c: &C) -> Self;
}

pub trait Inverse<Op, C>: Sized {
    fn inv(self, c: &C) -> Self;
}

pub trait InverseNonZero<Op, C>: Sized {
    fn inv(self, c: &C) -> Option<Self>;
}

pub trait CommutativeMonoid<Op, C>: Copy + CommutativeOp<Op, C> + Identity<Op, C> {
    fn exp<I: Natural>(self, n: I, cfg: &C) -> Self {
        if n == I::zero() {
            Identity::identity(cfg)
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

pub trait AbelianGroup<Op, C>: Sized + Copy + CommutativeMonoid<Op, C> + Inverse<Op, C> {}

pub mod ops {
    pub struct Add;
    pub struct Mul;
}

pub trait Field<C>:
    Sized
    + AbelianGroup<ops::Add, C>
    + CommutativeMonoid<ops::Mul, C>
    + InverseNonZero<ops::Mul, C>
    + Eq
{
    fn add(a: Self, b: Self, cfg: &C) -> Self {
        CommutativeOp::<ops::Add, C>::op(a, b, cfg)
    }
    fn sub(a: Self, b: Self, cfg: &C) -> Self {
        CommutativeOp::<ops::Add, C>::op(a, Inverse::<ops::Add, C>::inv(b, cfg), cfg)
    }
    fn mul(a: Self, b: Self, cfg: &C) -> Self {
        CommutativeOp::<ops::Mul, C>::op(a, b, cfg)
    }
    fn div(a: Self, b: Self, cfg: &C) -> Self {
        CommutativeOp::<ops::Mul, C>::op(
            a,
            InverseNonZero::<ops::Mul, C>::inv(b, cfg).unwrap(),
            cfg,
        )
    }
    fn zero(cfg: &C) -> Self {
        Identity::<ops::Add, C>::identity(cfg)
    }
    fn one(cfg: &C) -> Self {
        Identity::<ops::Mul, C>::identity(cfg)
    }
    fn two(cfg: &C) -> Self {
        let one = Self::one(cfg);
        Self::add(one, one, cfg)
    }
    fn pow<N: Natural>(self, n: N, cfg: &C) -> Self {
        CommutativeMonoid::<ops::Mul, C>::exp(self, n, cfg)
    }
    fn reciprocal(self, cfg: &C) -> Option<Self> {
        InverseNonZero::inv(self, cfg)
    }
    fn neg(self, cfg: &C) -> Self {
        Inverse::inv(self, cfg)
    }
}
