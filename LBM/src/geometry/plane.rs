use grid;
use super::Geometry;

pub struct Plane {
    n: (isize, isize),
    x: (usize, usize),
}

impl Plane {
    pub fn new(n: (isize, isize), x: (usize, usize)) -> Plane {
        Plane { n: n, x: x }
    }
}

impl Geometry for Plane {
    #[inline(always)]
    fn generate(&self, domain: &Array<FloatNum>) -> Array<FloatNum> {
        // top
        set_col(&mut domain, &constant::<FloatNum>(1.0, dim4!(nx)), 0);

        let mut bound = constant(1.0 as FloatNum, domain.dims());
        match (self.n.0, self.n.1, x.0, x.1) {
            (1, 0, x, _) => !(x > self.x.0),
            (0, 1, _, y) => !(y > self.x.1),
            (0, -1, _, y) => !(y < self.x.1),
            _ => unimplemented!(),
        }
    }
}