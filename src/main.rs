use std::ops;

use field::Field;
use rand::random;

mod field;
const MOD: u64 = 0x0014_4C3B_27FF;
type Zp = Field<MOD>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Point(Zp, Zp);

const A: Zp = Zp::new(100);
const B: Zp = Zp::new(1);
// https://en.wikipedia.org/wiki/List_of_prime_numbers

fn curve(x: Zp, y: Zp) -> bool {
    y * y == x * x * x + A * x + B
}

impl Point {
    pub fn new(x: Zp, y: Zp) -> Self {
        assert!(curve(x, y));
        Self(x, y)
    }
}

const G: Point = Point(Zp::new(2500), Zp::new(125001));

impl ops::Add for Point {
    type Output = Point;

    fn add(self, rhs: Self) -> Self::Output {
        let Point(x1, y1) = self;
        let Point(x2, y2) = rhs;
        assert!(!(x1 == x2 && y1 != y2));
        let l = if self != rhs {
            (y2 - y1) / (x2 - x1)
        } else {
            (Zp::new(3) * x1 * x1 + A) / (Zp::new(2) * y1)
        };
        let x3 = l * l - x1 - x2;
        let y3 = l * (x3 - x1) - y1;
        Point(x3, y3)
    }
}

impl ops::Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        let Point(x2, y2) = rhs;
        self + Point(x2, -y2)
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
struct PrivateKey(u128);

#[derive(Clone, Copy, Debug)]
struct PublicKey(Point);

fn gen_keys() -> (PrivateKey, PublicKey) {
    let pri = random();
    let pub_ = G * pri;
    (PrivateKey(pri), PublicKey(pub_))
}

impl PublicKey {
    pub fn encrypt(self, msg: Point) -> (Point, Point) {
        let t = random();
        let c1 = G * t;
        let c2 = self.0 * t + msg;
        (c1, c2)
    }
}

impl PrivateKey {
    pub fn decrypt(self, (c1, c2): (Point, Point)) -> Point {
        c2 - c1 * self.0
    }
}

fn main() {
    println!("{:?}", Zp::new(19381031) / Zp::new(312983120));
    println!("Hello, world! {:?}", gen_keys());
}

#[cfg(test)]
mod tests {
    use crate::{curve, gen_keys, Point, Zp, G};

    #[test]
    fn g_belongs() {
        assert!(curve(G.0, G.1));
    }

    #[test]
    fn back_forth() {
        let (pr, pb) = gen_keys();
        let msg = Point(Zp::new(232), Zp::new(3537));
        assert!(curve(msg.0, msg.1));
        let encrypted = pb.encrypt(msg);
        let decrypted = pr.decrypt(encrypted);
        assert_eq!(msg, decrypted);
    }

    #[test]
    fn points_add_itself() {
        let a = Point(Zp::new(232), Zp::new(3537));
        assert_eq!(a + a, Point(Zp::new(74095187791), Zp::new(9434911276)));
    }
}
