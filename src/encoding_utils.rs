use std::io::{BufRead, BufWriter, Cursor};

use crate::{
    algebra::{self, CommutativeOp, DiscreteRoot, Field, InitialPoint},
    base_traits::{Capacitor, FromRandom, Natural, RW},
    ecc::{PrivateKey, PublicKey},
    points_group::{Point, PointCfg},
};
use base64::prelude::*;
use rand::Rng;

fn bytes_to_point<F: Field + RW + DiscreteRoot<algebra::ops::Mul>, I: Natural + Sized>(
    bytes: &[u8],
    cfg: &PointCfg<F>,
    cap: usize,
) -> Point<F> {
    assert!(bytes.len() < F::LEN);
    let mut quintuple = vec![0u8; F::LEN];
    quintuple[0..bytes.len()].copy_from_slice(bytes);
    loop {
        let mut cur = Cursor::new(&quintuple);
        let x = F::from_bytes(&mut cur);
        if let Some(point) = Point::from_x(x, cfg) {
            return point;
        }
        quintuple[cap] += 1;
    }
}

pub fn text_to_points<F: Field + RW + DiscreteRoot<algebra::ops::Mul> + Capacitor, I: Natural>(
    text: &str,
    cfg: &PointCfg<F>,
) -> Vec<Point<F>>
where
    [(); F::LEN - 1]:,
{
    let bytes = text.as_bytes();

    let eff_length_incl_padding = F::capacity(&cfg.cf).min(F::LEN - 1) - 1;
    assert!(eff_length_incl_padding > 1);
    let iter_count = bytes.len() / eff_length_incl_padding;
    let mut res = vec![];
    for i in 0..iter_count {
        let chunk = &bytes[i * eff_length_incl_padding..(i + 1) * eff_length_incl_padding];
        res.push(bytes_to_point::<F, I>(chunk, cfg, eff_length_incl_padding));
    }
    if bytes.len() % eff_length_incl_padding != 0 {
        let chunk = &bytes[bytes.len() / eff_length_incl_padding * eff_length_incl_padding..];
        res.push(bytes_to_point::<F, I>(chunk, cfg, eff_length_incl_padding));
    }

    res
}

pub fn points_to_text<F: RW + Field>(points: impl Iterator<Item = Point<F>>, cap: usize) -> String {
    let mut bytes = vec![];
    let mut buf = vec![];
    for point in points {
        buf.clear();
        let b = point.x().to_bytes(&mut buf);
        for v in 0..b.min(cap) {
            if buf[v] == 0x00 {
                break;
            }
            bytes.push(buf[v]);
        }
    }
    String::from_utf8(bytes).unwrap()
}

pub fn points_to_base64<F: RW + Field>(points: impl Iterator<Item = Point<F>>) -> String {
    let mut v = vec![];
    for p in points {
        p.to_bytes(&mut v);
    }
    BASE64_STANDARD.encode(&v)
}

pub fn base64_to_points<F: RW + Field>(base64: &str) -> Vec<Point<F>>
where
    [(); Point::<F>::LEN]:,
{
    let bytes = BASE64_STANDARD.decode(base64).unwrap();
    assert_eq!(bytes.len() % Point::<F>::LEN, 0);
    let mut cur = Cursor::new(&bytes);
    let mut res = vec![];
    while !cur.is_empty() {
        res.push(Point::<F>::from_bytes(&mut cur));
    }
    res
}

pub fn encrypt_message_and_encode<
    F: Field + RW + DiscreteRoot<algebra::ops::Mul> + Capacitor,
    I: FromRandom<()> + Natural,
>(
    key: PublicKey<Point<F>>,
    msg: &str,
    rng: &mut impl Rng,
    cfg: &PointCfg<F>,
) -> String
where
    [(); F::LEN - 1]:,
{
    let points = text_to_points::<F, I>(msg, cfg);
    let encrypted = points
        .iter()
        .flat_map(|p| {
            let (c1, c2) = key.encrypt::<I>(*p, rng, cfg);
            [c1, c2]
        })
        .collect::<Vec<_>>();
    points_to_base64(encrypted.into_iter())
}

pub fn decode_message_and_decrypt<IP: RW + Natural, F: RW + Field + Capacitor>(
    key: PrivateKey<IP>,
    msg_base64: &str,
    cfg: &PointCfg<F>,
) -> String
where
    [(); F::LEN]:,
    [(); Point::<F>::LEN]:,
{
    let points = base64_to_points::<F>(msg_base64);
    assert!(points.len() % 2 == 0);
    let decrypted = points
        .iter()
        .array_chunks::<2>()
        .map(|[c1, c2]| key.decrypt((*c1, *c2), cfg))
        .collect::<Vec<_>>();
    points_to_text(decrypted.into_iter(), F::capacity(&cfg.cf) - 1)
}

#[cfg(test)]
mod tests {

    use rand::SeedableRng;

    use crate::{
        base_traits::Capacitor,
        ecc::gen_keys,
        mod_field::{ModField, ModFieldCfg},
        points_group::{Point, PointCfg},
        points_to_text, text_to_points,
    };

    use super::{decode_message_and_decrypt, encrypt_message_and_encode};

    fn config() -> PointCfg<ModField<u64>> {
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
        cfg_group
    }

    const TEXTS: [&str; 4] = [
        "Hello, world",
        "Aaa",
        "A very long sentence actually, yeah",
        "Hello, world!! :)",
    ];

    #[test]
    fn text2points2text() {
        let cfg_group = config();
        for text in TEXTS {
            let points = text_to_points::<_, u64>(text, &cfg_group);
            let text2 = points_to_text(
                points.iter().copied(),
                ModField::<u64>::capacity(&cfg_group.cf) - 1,
            );
            assert_eq!(text, text2);
        }
    }

    #[test]
    fn encrypt_encode_decode_decrypt() {
        let cfg_group = config();
        let mut gen = rand_chacha::ChaCha8Rng::from_seed([1u8; 32]);
        let (pr, pb) = gen_keys::<_, u128, _>(&mut gen, &cfg_group);
        for text in TEXTS {
            for _ in 0..10 {
                let secret = encrypt_message_and_encode::<_, u64>(pb, text, &mut gen, &cfg_group);
                let decoded = decode_message_and_decrypt(pr, &secret, &cfg_group);
                assert_eq!(text, decoded);
            }
        }
    }
}
