use std::io::{Read, Write};

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

pub struct PointCfg<F: Field> {
    pub g: Point<F>,
    pub a: F,
    pub b: F,
    pub cf: F::Cfg,
}

impl<F: Field> Configurable for Point<F> {
    type Cfg = PointCfg<F>;
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
        y2.sqrt(&cp.cf).map(|yres| Self::new(x, yres, cp))
    }
}

impl<F: Field + DiscreteRoot<algebra::ops::Mul>> Point<F>
where
    F: FromRandom<F::Cfg>,
{
    pub fn random<R: Rng>(r: &mut R, cfg: &<Self as Configurable>::Cfg) -> Self {
        loop {
            let x = F::random(r, &cfg.cf);
            if let Some(p) = Self::from_x(x, cfg) {
                return p;
            }
        }
    }
}

impl<F: RW + Field> RW for Point<F> {
    fn to_bytes(self, w: &mut impl Write) -> usize {
        self.x.to_bytes(w) + self.y.to_bytes(w)
    }

    fn from_bytes(r: &mut impl Read) -> Self {
        Self::new_unsafe(F::from_bytes(r), F::from_bytes(r))
    }

    const LEN: usize = F::LEN * 2;
}

impl<F: Field> InitialPoint<Point<F>> for PointCfg<F> {
    fn g(&self) -> Point<F> {
        self.g
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        algebra::{self, CommutativeOp},
        mod_field::{ModField, ModFieldCfg},
    };

    use super::{Point, PointCfg};

    fn cfg() -> PointCfg<ModField<u64>> {
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

    #[test]
    fn g_exists() {
        let cfg = cfg();
        Point::new(cfg.g.x(), cfg.g.y(), &cfg);
    }

    fn p(x: u64, y: u64) -> Point<ModField<u64>> {
        Point::new(
            ModField::new(x, &cfg().cf),
            ModField::new(y, &cfg().cf),
            &cfg(),
        )
    }

    #[test]
    fn points_add_itself() {
        let a = p(232, 3537);
        assert_eq!(
            CommutativeOp::<algebra::ops::Add>::op(a, a, &cfg()),
            p(74095187791, 9434911276)
        );
    }

    // http://christelbach.com/ECCalculator.aspx
    #[test]
    fn points_add_two() {
        let a = p(82226830584, 16727101863);
        let b = p(17120951320, 15809323217);
        assert_eq!(
            CommutativeOp::<algebra::ops::Add>::op(a, b, &cfg()),
            p(3851261364, 66206903692)
        );
    }
}
