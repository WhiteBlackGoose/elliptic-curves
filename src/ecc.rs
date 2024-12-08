use std::ops;

use base64::{prelude::BASE64_STANDARD, Engine};
use rand::Rng;

use crate::{
    algebra::{self, CommutativeOp, InitialPoint, Inverse},
    base_traits::{FromRandom, Natural, RW},
    mod_field::ModField,
    points_group::{Point, PointCfg},
};

#[derive(Clone, Copy, Debug)]
pub struct PrivateKey<I>(I);

#[derive(Clone, Copy, Debug)]
pub struct PublicKey<P>(P);

fn rand_u128(r: &mut impl Rng) -> u128 {
    ((r.next_u64() as u128) << 64) + r.next_u64() as u128
}

pub fn gen_keys<R: Rng, I: FromRandom + Natural, P: CommutativeOp<algebra::ops::Add>>(
    r: &mut R,
    g: P,
    cfg: &P::Cfg,
) -> (PrivateKey<I>, PublicKey<P>) {
    let pri = I::random(r);
    let pub_ = P::exp(g, pri, cfg);
    (PrivateKey(pri), PublicKey(pub_))
}

impl<P: CommutativeOp<algebra::ops::Add> + RW, I: Natural + FromRandom> PublicKey<P>
where
    <P as algebra::Configurable>::Cfg: InitialPoint<P>,
{
    pub fn encrypt(self, msg: P, rng: &mut impl Rng, cfg: &P::Cfg) -> (P, P) {
        let t = I::random(rng);
        let c1 = P::exp(InitialPoint::g(cfg), t, cfg);
        let c2 = P::op(P::exp(self.0, t, cfg), msg, cfg);
        (c1, c2)
    }

    pub fn base64(self) -> String {
        self.0.to_base64()
    }

    pub fn from_base64(base64: &str) -> Self {
        Self(P::from_base64(base64))
    }
}

impl<I: Natural + RW> PrivateKey<I> {
    pub fn decrypt<P: CommutativeOp<algebra::ops::Add> + Inverse<algebra::ops::Add>>(
        self,
        (c1, c2): (P, P),
        cfg: &P::Cfg,
    ) -> P {
        P::op(c2, P::inv(P::exp(c1, self.0, cfg), cfg), cfg)
    }

    pub fn base64(self) -> String {
        self.0.to_base64()
    }

    pub fn from_base64(base64: &str) -> Self {
        Self(I::from_base64(base64))
    }
}

pub fn bytes_to_zp(input: &[u8]) -> Zp {
    let mut bytes = [0x00u8; 8];
    bytes[0..input.len()].copy_from_slice(input);
    Zp::new(u64::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {

    #[test]
    fn back_forth() {
        let mut gen = rand_chacha::ChaCha8Rng::from_seed([1u8; 32]);
        for _ in 0..100 {
            let (pr, pb) = gen_keys(&mut gen);
            let msg = Point::random(&mut gen);
            let encrypted = pb.encrypt(msg, &mut gen);
            let decrypted = pr.decrypt(encrypted);
            assert_eq!(msg, decrypted);
        }
    }
}
