use crate::ecc::{bytes_to_zp, Point, PrivateKey, PublicKey};
use base64::prelude::*;
use rand::Rng;

fn bytes_to_point(bytes: &[u8]) -> Point {
    assert!(bytes.len() <= 4);
    let mut quintuple = [0u8; 5];
    quintuple[0..bytes.len()].copy_from_slice(bytes);
    loop {
        let x = bytes_to_zp(&quintuple);
        if let Some(point) = Point::from_x(x) {
            return point;
        }
        quintuple[4] += 1;
    }
}

pub fn text_to_points(text: &str) -> Vec<Point> {
    let bytes = text.as_bytes();
    let mut iter = bytes.iter().copied().array_chunks::<4>();
    let mut res = vec![];
    for chunk in iter.by_ref() {
        res.push(bytes_to_point(&chunk));
    }
    if let Some(leftover) = iter.into_remainder() {
        res.push(bytes_to_point(&leftover.collect::<Vec<_>>()));
    }
    res
}

fn point_to_bytes(point: Point) -> [u8; 8] {
    let x = point.x().nat();
    x.to_le_bytes()
}

pub fn points_to_text(points: impl Iterator<Item = Point>) -> String {
    let mut bytes = vec![];
    for point in points {
        let b = point_to_bytes(point);
        for v in b.iter().take(4) {
            if *v == 0x00 {
                break;
            }
            bytes.push(*v);
        }
    }
    String::from_utf8(bytes).unwrap()
}

pub fn points_to_base64(points: impl Iterator<Item = Point>) -> String {
    let bytes = points.flat_map(|p| p.bytes()).collect::<Vec<_>>();
    BASE64_STANDARD.encode(bytes)
}

pub fn base64_to_points(base64: &str) -> Vec<Point> {
    let bytes = BASE64_STANDARD.decode(base64).unwrap();
    assert_eq!(bytes.len() % 16, 0);
    bytes
        .iter()
        .copied()
        .array_chunks::<16>()
        .map(Point::from_bytes)
        .collect()
}

pub fn encrypt_message_and_encode(key: PublicKey, msg: &str, rng: &mut impl Rng) -> String {
    let points = text_to_points(msg);
    let encrypted = points.iter().flat_map(|p| {
        let (c1, c2) = key.encrypt(*p, rng);
        [c1, c2]
    });
    points_to_base64(encrypted)
}

pub fn decode_message_and_decrypt(key: PrivateKey, msg_base64: &str, rng: &mut impl Rng) -> String {
    let points = base64_to_points(msg_base64);
    assert!(points.len() % 2 == 0);
    let decrypted = points
        .iter()
        .array_chunks::<2>()
        .map(|[c1, c2]| key.decrypt((*c1, *c2)));
    points_to_text(decrypted)
}

#[cfg(test)]
mod tests {

    use crate::{points_to_text, text_to_points};

    #[test]
    fn text2points2text() {
        let text = "Hello, world";
        let points = text_to_points(text);
        let text2 = points_to_text(points.iter().copied());
        assert_eq!(text, text2);
    }
}
