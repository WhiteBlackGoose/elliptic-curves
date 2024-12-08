use std::ops::*;

use rand::Rng;

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

pub trait FromRandom {
    fn random(rng: &mut impl Rng) -> Self;
}

impl Natural for u64 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }

    fn max() -> Self {
        u64::MAX
    }
}

impl FromRandom for u64 {
    fn random(rng: &mut impl Rng) -> Self {
        rng.next_u64()
    }
}

pub trait RW {
    const LEN: usize;

    fn to_bytes(self) -> [u8; Self::LEN];
    fn from_bytes(bytes: [u8; Self::LEN]) -> Self;
}
