use rand::Rng;

use crate::{
    algebra::{self, CommutativeOp, Field, InitialPoint, Inverse},
    base_traits::{FromRandom, Natural, RW},
    mod_field::{ModField, ModFieldCfg},
    points_group::{Point, PointCfg},
};

#[derive(Clone, Copy, Debug)]
pub struct PrivateKey<I>(I);

#[derive(Clone, Copy, Debug)]
pub struct PublicKey<P>(P);

pub fn gen_keys<R: Rng, I: FromRandom<()> + Natural, P: CommutativeOp<algebra::ops::Add>>(
    r: &mut R,
    cfg: &P::Cfg,
) -> (PrivateKey<I>, PublicKey<P>)
where
    P::Cfg: InitialPoint<P>,
{
    let pri = I::random(r, &());
    let pub_ = P::exp(cfg.g(), pri, cfg);
    (PrivateKey(pri), PublicKey(pub_))
}

impl<P: CommutativeOp<algebra::ops::Add> + RW> PublicKey<P>
where
    <P as algebra::Configurable>::Cfg: InitialPoint<P>,
{
    pub fn encrypt<I: Natural + FromRandom<()>>(
        self,
        msg: P,
        rng: &mut impl Rng,
        cfg: &P::Cfg,
    ) -> (P, P) {
        let t = I::random(rng, &());
        // C1 = t * G
        let c1 = P::exp(InitialPoint::g(cfg), t, cfg);
        // C2 = t * Pub + msg
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
        // C2 + -(priv * C1)
        // = t * Pub + msg - priv * t * G
        // = t * priv * G + msg - priv * t * G
        // = msg
        P::op(c2, P::inv(P::exp(c1, self.0, cfg), cfg), cfg)
    }

    pub fn base64(self) -> String {
        self.0.to_base64()
    }

    pub fn from_base64(base64: &str) -> Self {
        Self(I::from_base64(base64))
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use crate::{
        ecc::gen_keys,
        mod_field::{ModField, ModFieldCfg},
        points_group::{Point, PointCfg},
    };

    #[test]
    fn back_forth() {
        let cfg_field = ModFieldCfg {
            rem: 0x0014_4C3B_27FFu64,
            // 0x1FFF_FFFF_FFFF_FFFF
        };
        let cfg_group = PointCfg {
            g: Point::new_unsafe(
                ModField::new(2500, &cfg_field),
                ModField::new(125001, &cfg_field),
            ),
            a: ModField::new(100, &cfg_field),
            b: ModField::new(1, &cfg_field),
            cf: cfg_field,
        };
        let mut gen = rand_chacha::ChaCha8Rng::from_seed([1u8; 32]);
        for _ in 0..100 {
            let (pr, pb) = gen_keys::<_, u128, _>(&mut gen, &cfg_group);
            // let msg = Point::random(&mut gen, &cfg_group);
            let msg = Point::new(
                ModField::new(369344026516415816, &cfg_group.cf),
                ModField::new(20868581830, &cfg_group.cf),
                &cfg_group,
            );
            let encrypted = pb.encrypt::<u128>(msg, &mut gen, &cfg_group);
            let decrypted = pr.decrypt(encrypted, &cfg_group);
            assert_eq!(msg, decrypted);
        }
    }
}
