use arrayfire::*; 
use crate::FloatNum;
// use crate::grid;
use crate::traits::Geometry;

pub struct Circle {
    x_c: FloatNum,
    y_c: FloatNum,
    r: FloatNum,
}

impl Circle {
    pub fn new(lx: usize, ly: usize) -> Self {
        let lx = lx as FloatNum;
        let ly = ly as FloatNum;
        Self {
            x_c: lx / 2. - 0.2 * lx,
            y_c: ly / 2.,
            r: 0.125 * ly,
        }
    }
}

impl Geometry for Circle {
    #[inline(always)]
    fn generate(&self, domain: &Array<FloatNum>) -> Array<FloatNum> {
        let r = constant::<FloatNum>(self.r as FloatNum, domain.dims());
        let r_sq = &r * &r;
        let circle = moddims(
            &le(
                &(pow(
                    &(flat(&x) - obstacle_x as FloatNum),
                    &(2.0 as FloatNum),
                    false,
                ) + pow(
                    &(flat(&y) - obstacle_y as FloatNum),
                    &(2.0 as FloatNum),
                    false,
                )),
                &flat(&r_sq),
                false,
            ),
            dims,
        );

        selectr(domain, &circle, 0.0 as f64)
    }
}