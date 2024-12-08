use std::ops;

use base64::{prelude::BASE64_STANDARD, Engine};
use rand::Rng;

use crate::field::Field;

const MOD: u64 = 0x0014_4C3B_27FF;
pub type Zp = Field<MOD>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    x: Zp,
    y: Zp,
}

const A: Zp = Zp::new(100);
const B: Zp = Zp::new(1);
// https://en.wikipedia.org/wiki/List_of_prime_numbers

fn curve(x: Zp, y: Zp) -> bool {
    y * y == x * x * x + A * x + B
}

impl Point {
    pub fn x(self) -> Zp {
        self.x
    }

    pub fn y(self) -> Zp {
        self.y
    }

    pub fn new(x: Zp, y: Zp) -> Self {
        assert!(curve(x, y), "Invalid point: x: {}, y: {}", x, y);
        Self { x, y }
    }

    pub fn validate(self) -> Result<(), ()> {
        if curve(self.x, self.y) {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn from_x(x: Zp) -> Option<Self> {
        let y2 = x.pow(3) + A * x + B;
        if let Some(yres) = y2.sqrt() {
            Some(Self::new(x, yres))
        } else {
            None
        }
    }

    pub fn random<R: Rng>(r: &mut R) -> Self {
        loop {
            let x = Zp::random(r);
            if let Some(p) = Self::from_x(x) {
                return p;
            }
        }
    }

    pub fn bytes(self) -> [u8; 16] {
        let mut res = [0u8; 16];
        res[..8].copy_from_slice(&self.x.nat().to_le_bytes());
        res[8..].copy_from_slice(&self.y.nat().to_le_bytes());
        res
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        let x = u64::from_le_bytes(bytes[..8].try_into().unwrap());
        let y = u64::from_le_bytes(bytes[8..].try_into().unwrap());
        Self::new(Zp::new(x), Zp::new(y))
    }
}

const G: Point = Point {
    x: Zp::new(2500),
    y: Zp::new(125001),
};

impl ops::Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        let Point { x: x1, y: y1 } = self;
        let Point { x: x2, y: y2 } = rhs;
        assert!(!(x1 == x2 && y1 != y2));
        let l = if self != rhs {
            (y2 - y1) / (x2 - x1)
        } else {
            (Zp::new(3) * x1 * x1 + A) / (Zp::new(2) * y1)
        };
        let x3 = l * l - x1 - x2;
        let y3 = -(l * (x3 - x1) + y1);
        Point::new(x3, y3)
    }
}

impl ops::Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let Point { x: x2, y: y2 } = rhs;
        self + Point::new(x2, -y2)
    }
}

impl ops::Mul<u128> for Point {
    type Output = Point;

    fn mul(self, rhs: u128) -> Self::Output {
        assert!(rhs > 0);
        if rhs == 1 {
            return self;
        }
        let m1 = rhs / 2;
        let pr = self * m1;
        match rhs % 2 {
            0 => pr + pr,
            _ => pr + pr + self,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PrivateKey(u128);

#[derive(Clone, Copy, Debug)]
pub struct PublicKey(Point);

fn rand_u128(r: &mut impl Rng) -> u128 {
    ((r.next_u64() as u128) << 64) + r.next_u64() as u128
}

pub fn gen_keys<R: Rng>(r: &mut R) -> (PrivateKey, PublicKey) {
    let pri = rand_u128(r);
    let pub_ = G * pri;
    (PrivateKey(pri), PublicKey(pub_))
}

impl PublicKey {
    pub fn encrypt(self, msg: Point, rng: &mut impl Rng) -> (Point, Point) {
        let t = rand_u128(rng);
        let c1 = G * t;
        let c2 = self.0 * t + msg;
        (c1, c2)
    }

    pub fn base64(self) -> String {
        let mut bytes = [0u8; 16];
        bytes[..8].copy_from_slice(&self.0.x.nat().to_le_bytes());
        bytes[8..].copy_from_slice(&self.0.x.nat().to_le_bytes());
        BASE64_STANDARD.encode(bytes)
    }

    pub fn from_base64(base64: &str) -> Self {
        let bytes = BASE64_STANDARD.decode(base64).unwrap();
        let x = Zp::new(u64::from_le_bytes(bytes[..8].try_into().unwrap()));
        let y = Zp::new(u64::from_le_bytes(bytes[8..].try_into().unwrap()));
        Self(Point::new(x, y))
    }
}

impl PrivateKey {
    pub fn decrypt(self, (c1, c2): (Point, Point)) -> Point {
        c2 - c1 * self.0
    }

    pub fn base64(self) -> String {
        BASE64_STANDARD.encode(self.0.to_le_bytes())
    }

    pub fn from_base64(base64: &str) -> Self {
        Self(u128::from_le_bytes(
            BASE64_STANDARD.decode(base64).unwrap().try_into().unwrap(),
        ))
    }
}

pub fn bytes_to_zp(input: &[u8]) -> Zp {
    let mut bytes = [0x00u8; 8];
    bytes[0..input.len()].copy_from_slice(input);
    Zp::new(u64::from_le_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use crate::ecc::{gen_keys, Point, Zp, G};

    #[test]
    fn g_belongs() {
        assert!(G.validate().is_ok());
    }

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

    #[test]
    fn points_add_itself() {
        let a = Point::new(Zp::new(232), Zp::new(3537));
        assert_eq!(a + a, Point::new(Zp::new(74095187791), Zp::new(9434911276)));
    }

    // http://christelbach.com/ECCalculator.aspx
    #[test]
    fn points_add_two() {
        let a = Point::new(Zp::new(82226830584), Zp::new(16727101863));
        let b = Point::new(Zp::new(17120951320), Zp::new(15809323217));
        assert_eq!(a + b, Point::new(Zp::new(3851261364), Zp::new(66206903692)));
    }
}
