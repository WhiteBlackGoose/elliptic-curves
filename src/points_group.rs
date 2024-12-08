use crate::algebra::{self, CommutativeOp, Configurable, Field};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point<F> {
    x: F,
    y: F,
}

pub struct PointCfg<CFG, F> {
    g: Point<F>,
    a: F,
    b: F,
    cf: CFG,
}

impl<F: Field> Configurable for Point<F> {
    type Cfg = PointCfg<F::Cfg, F>;
}

impl<F: Field> Point<F> {
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
