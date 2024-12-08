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

pub trait FromRandom<C> {
    fn random(rng: &mut impl Rng, cfg: &C) -> Self;
}

impl<T> FromRandom<T> for u64 {
    fn random(rng: &mut impl Rng, _: &T) -> Self {
        rng.next_u64()
    }
}

impl<T> FromRandom<T> for u128 {
    fn random(rng: &mut impl Rng, _: &T) -> Self {
        ((rng.next_u64() as u128) << 64) + rng.next_u64() as u128
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

macro_rules! impl_stuff {
    ($ty:ident) => {
        impl RW for $ty {
            // -1 so we reserve one byte for padding
            const LEN: usize = size_of::<Self>() - 1;

            fn to_bytes(self, w: &mut impl Write) -> usize {
                w.write(&self.to_le_bytes()).unwrap()
            }

            fn from_bytes(r: &mut impl Read) -> Self {
                let mut buf = vec![0u8; size_of::<Self>()];
                r.read_exact(&mut buf).unwrap();
                Self::from_le_bytes(buf.try_into().unwrap())
            }
        }

        impl Natural for $ty {
            fn zero() -> Self {
                0
            }

            fn one() -> Self {
                1
            }

            fn max() -> Self {
                $ty::MAX
            }
        }
    };
}

impl_stuff!(u64);
impl_stuff!(u128);
impl_stuff!(u8);
