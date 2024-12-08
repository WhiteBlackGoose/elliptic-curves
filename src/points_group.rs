use crate::{algebra::Field, base_traits::Natural, mod_field::ModFieldCfg};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point<F> {
    x: F,
    y: F,
}

pub struct PointCfg<F> {
    g: Point<F>,
    a: F,
    b: F,
}

impl<F: Field> Point<F> {
    pub fn new(x: F, y: F, cf: &F::Cfg, cp: &PointCfg<F>) -> Self {
        let lhs = y.sqr(cf);
        let rhs = F::add(F::add(x.cube(cf), F::mul(cp.a, x, cf), cf), cp.b, cf);
        assert!(lhs == rhs);
        Self { x, y }
    }
}
