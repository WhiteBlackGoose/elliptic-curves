use std::ops::*;

pub trait Natural:
    Sized
    + Copy
    + std::ops::Add<Output = Self>
    + std::ops::Mul<Output = Self>
    + Div<Output = Self>
    + Rem<Output = Self>
    + Sub<Output = Self>
    + Eq
    + Ord
{
    fn zero() -> Self;
    fn one() -> Self;
    fn two() -> Self {
        Self::one() + Self::one()
    }
    fn max() -> Self;
}

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
    fn exp<I: Natural>(self, cfg: &C, n: I) -> Self {
        if n == I::zero() {
            Identity::identity(cfg)
        } else if n == I::one() {
            self
        } else {
            let m = n / I::two();
            let r = self.exp(cfg, n);
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
}
