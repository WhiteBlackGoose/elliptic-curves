use std::io::{BufRead, BufWriter, Cursor};

use crate::{
    algebra::{self, CommutativeOp, DiscreteRoot, Field, InitialPoint},
    base_traits::{FromRandom, Natural, RW},
    ecc::{PrivateKey, PublicKey},
    points_group::{Point, PointCfg},
};
use base64::prelude::*;
use rand::Rng;

fn bytes_to_point<F: Field + RW + DiscreteRoot<algebra::ops::Mul>, I: Natural + Sized>(
    bytes: &[u8],
    cfg: &PointCfg<F>,
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
        quintuple[F::LEN - 1] += 1;
    }
}

pub fn text_to_points<F: Field + RW + DiscreteRoot<algebra::ops::Mul>, I: Natural>(
    text: &str,
    cfg: &PointCfg<F>,
) -> Vec<Point<F>>
where
    [(); F::LEN - 1]:,
{
    let bytes = text.as_bytes();
    // -1 so we reserve one byte for padding
    let mut iter = bytes.iter().copied().array_chunks::<{ F::LEN - 1 }>();
    let mut res = vec![];
    for chunk in iter.by_ref() {
        res.push(bytes_to_point::<F, I>(&chunk, cfg));
    }
    if let Some(leftover) = iter.into_remainder() {
        res.push(bytes_to_point::<F, I>(&leftover.collect::<Vec<_>>(), cfg));
    }
    res
}

pub fn points_to_text<F: RW + Field>(points: impl Iterator<Item = Point<F>>) -> String {
    let mut bytes = vec![];
    let mut buf = vec![];
    for point in points {
        buf.clear();
        let b = point.x().to_bytes(&mut buf);
        for v in 0..b.min(F::LEN - 1) {
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
    F: Field + RW + DiscreteRoot<algebra::ops::Mul>,
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
    let encrypted = points.iter().flat_map(|p| {
        let (c1, c2) = key.encrypt::<I>(*p, rng, cfg);
        [c1, c2]
    });
    points_to_base64(encrypted)
}

pub fn decode_message_and_decrypt<I: RW + Natural, F: RW + Field>(
    key: PrivateKey<I>,
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
        .map(|[c1, c2]| key.decrypt((*c1, *c2), cfg));
    points_to_text(decrypted)
}

#[cfg(test)]
mod tests {

    use crate::{
        mod_field::{ModField, ModFieldCfg},
        points_group::{Point, PointCfg},
        points_to_text, text_to_points,
    };

    #[test]
    fn text2points2text() {
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
        let texts = [
            "Hello, world",
            "Aaa",
            "A very long sentence actually, yeah",
            "Hello, world!! :)",
        ];
        for text in texts {
            let points = text_to_points::<_, u64>(text, &cfg_group);
            let text2 = points_to_text(points.iter().copied());
            assert_eq!(text, text2);
        }
    }
}
