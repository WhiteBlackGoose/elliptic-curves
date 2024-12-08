use rand::Rng;

use crate::{
    algebra::{
        self, CommutativeMonoid, CommutativeOp, Configurable, DiscreteRoot, Field, Identity,
        InitialPoint, Inverse,
    },
    base_traits::{FromRandom, RW},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point<F> {
    x: F,
    y: F,
}

pub struct PointCfg<CFG, F> {
    pub g: Point<F>,
    pub a: F,
    pub b: F,
    pub cf: CFG,
}

impl<F: Field> Configurable for Point<F> {
    type Cfg = PointCfg<F::Cfg, F>;
}

impl<F: Field> Point<F> {
    pub fn new_unsafe(x: F, y: F) -> Self {
        Self { x, y }
    }

    pub fn new(x: F, y: F, cp: &<Self as Configurable>::Cfg) -> Self {
        let lhs = y.sqr(&cp.cf);
        let rhs = F::add(
            F::add(x.cube(&cp.cf), F::mul(cp.a, x, &cp.cf), &cp.cf),
            cp.b,
            &cp.cf,
        );
        assert!(lhs == rhs);
        Self { x, y }
    }
}

impl<F: Field> CommutativeOp<algebra::ops::Add> for Point<F> {
    fn op(a: Self, b: Self, c: &Self::Cfg) -> Self {
        let Point { x: x1, y: y1 } = a;
        let Point { x: x2, y: y2 } = b;
        assert!(!(x1 == x2 && y1 != y2));
        let l = if a != b {
            F::div(F::sub(y2, y1, &c.cf), F::sub(x2, x1, &c.cf), &c.cf)
        } else {
            // (3x^2 + a) / (2y)
            F::div(
                F::add(F::mul(F::three(&c.cf), x1.sqr(&c.cf), &c.cf), c.a, &c.cf),
                F::mul(F::two(&c.cf), y1, &c.cf),
                &c.cf,
            )
        };
        let x3 = F::sub(l.sqr(&c.cf), F::add(x1, x2, &c.cf), &c.cf);
        let y3 = F::neg(
            F::add(F::mul(l, F::sub(x3, x1, &c.cf), &c.cf), y1, &c.cf),
            &c.cf,
        );
        Point::new(x3, y3, c)
    }
}

impl<F: Field> Inverse<algebra::ops::Add> for Point<F> {
    fn inv(self, c: &Self::Cfg) -> Self {
        Self {
            x: self.x,
            y: F::neg(self.y, &c.cf),
        }
    }
}

impl<F> Point<F> {
    pub fn x(self) -> F {
        self.x
    }

    pub fn y(self) -> F {
        self.y
    }
}

impl<F: Field + DiscreteRoot<algebra::ops::Mul>> Point<F> {
    pub fn from_x(x: F, cp: &<Self as Configurable>::Cfg) -> Option<Self> {
        let y2 = F::add(
            F::add(x.cube(&cp.cf), F::mul(cp.a, x, &cp.cf), &cp.cf),
            cp.b,
            &cp.cf,
        );
        if let Some(yres) = y2.sqrt(&cp.cf) {
            Some(Self::new(x, yres, cp))
        } else {
            None
        }
    }
}

impl<F: Field + FromRandom + DiscreteRoot<algebra::ops::Mul>> Point<F> {
    pub fn random<R: Rng>(r: &mut R, cfg: &<Self as Configurable>::Cfg) -> Self {
        loop {
            let x = F::random(r);
            if let Some(p) = Self::from_x(x, cfg) {
                return p;
            }
        }
    }
}

impl<F: RW> RW for Point<F> {
    const LEN: usize = 2 * F::LEN;

    fn to_bytes(self) -> [u8; Self::LEN] {
        let mut res = [0u8; Self::LEN];
        res[..F::LEN].copy_from_slice(&self.x.to_bytes());
        res[F::LEN..].copy_from_slice(&self.y.to_bytes());
        res
    }

    fn from_bytes(bytes: [u8; Self::LEN]) -> Self {
        let x = u64::from_le_bytes(bytes[..8].try_into().unwrap());
        let y = u64::from_le_bytes(bytes[8..].try_into().unwrap());
        Self::new(Zp::new(x), Zp::new(y))
    }
}

impl<C, F: Copy> InitialPoint<Point<F>> for PointCfg<C, F> {
    fn g(&self) -> Point<F> {
        self.g
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

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
