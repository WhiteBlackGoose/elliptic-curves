use crate::ecc::{bytes_to_zp, Point};
use base64::prelude::*;

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

pub fn points_to_text(points: &[Point]) -> String {
    let mut bytes = vec![];
    for point in points {
        let b = point_to_bytes(*point);
        for v in b.iter().take(4) {
            if *v == 0x00 {
                break;
            }
            bytes.push(*v);
        }
    }
    String::from_utf8(bytes).unwrap()
}

pub fn points_to_base64(points: &[Point]) -> String {
    let bytes = points.iter().flat_map(|p| p.bytes()).collect::<Vec<_>>();
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
