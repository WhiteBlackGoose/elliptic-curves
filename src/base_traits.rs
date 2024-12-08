use std::{
    io::{Cursor, Read, Write},
    ops::*,
};

use base64::prelude::*;
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

pub trait RW: Sized {
    const LEN: usize;

    fn to_bytes(self, w: &mut impl Write) -> usize;
    fn from_bytes(r: &mut impl Read) -> Self;

    fn to_base64(self) -> String {
        let mut buf = vec![];
        let len = self.to_bytes(&mut buf);
        BASE64_STANDARD.encode(&buf[..len])
    }

    fn from_base64(base64: &str) -> Self {
        let decoded = BASE64_STANDARD.decode(base64).unwrap();
        let mut cur = Cursor::new(&decoded);
        Self::from_bytes(&mut cur)
    }
}
