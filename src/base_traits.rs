use std::{
    io::{Cursor, Read, Write},
    ops::*,
};

use base64::prelude::*;
use primitive_types::U256;
use rand::Rng;

use crate::algebra::Configurable;

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
        rng.next_u64().wrapping_mul(0x1983018027498101)
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

pub trait Capacitor: Configurable {
    /// how many bytes it can efficiently store
    fn capacity(cfg: &Self::Cfg) -> usize;
}

macro_rules! impl_stuff {
    ($ty:ident) => {
        impl RW for $ty {
            const LEN: usize = size_of::<Self>();

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

impl Natural for U256 {
    fn zero() -> Self {
        U256::zero()
    }

    fn one() -> Self {
        U256::one()
    }

    fn max() -> Self {
        U256::MAX
    }
}

impl RW for U256 {
    const LEN: usize = size_of::<U256>();

    fn to_bytes(self, w: &mut impl Write) -> usize {
        w.write(&self.to_little_endian()).unwrap()
    }

    fn from_bytes(r: &mut impl Read) -> Self {
        let mut buf = vec![0u8; size_of::<Self>()];
        r.read_exact(&mut buf).unwrap();
        Self::from_little_endian(&buf)
    }
}

impl FromRandom<()> for U256 {
    fn random(rng: &mut impl Rng, cfg: &()) -> Self {
        let l1: U256 = u128::random(rng, cfg).into();
        let l2: U256 = u128::random(rng, cfg).into();
        (l1 << 128) + l2
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::RW;

    #[test]
    fn data_persistance() {
        let n: u128 = 101793696879097904749597416266766297740;
        let mut buf = vec![];
        n.to_bytes(&mut buf);
        let mut cur = Cursor::new(&buf);
        let c = u128::from_bytes(&mut cur);
        assert_eq!(n, c);
    }
}
