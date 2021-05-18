use arrayfire::*;

#[repr(C)]
pub struct LidSimulation {
    total_nodes: u64,
    nx: u64,
    ny: u64,
    omega: FloatNum,
    dims: Dim4,
    dims_rgba: Dim4,
    colors: Array<u8>,
    f: Array<FloatNum>,
    nb_index: Array<FloatNum>,
    zeroed_on: Array<FloatNum>,
    to_reflect: Array<u64>,
    reflected: Array<u64>,
    density: Array<FloatNum>,
    ux: Array<FloatNum>,
    uy: Array<FloatNum>,
    on: Array<u32>,
    bound: Array<FloatNum>,
    ex: Array<FloatNum>,
    ey: Array<FloatNum>,
    eu: Array<FloatNum>,
    w: Array<FloatNum>,
    u_sq: Array<FloatNum>,
    ux_lid_af: Array<FloatNum>,
    uy_lid_af: Array<FloatNum>,
}

impl Simulation for LidSimulation {
    pub fn set_boundary_conditions(&mut self) {
        // circle
        let mut bound = constant::<FloatNum>(1.0, dims);
        let r = constant::<FloatNum>(obstacle_r as FloatNum, dims);
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
        bound = selectr(&bound, &circle, 0.0 as f64);
        set_col(&mut bound, &constant::<FloatNum>(1.0, dim4!(nx)), 0); //top
        set_col(
            &mut bound,
            &constant::<FloatNum>(1.0, dim4!(nx)),
            ny as i64 - 1,
        ); //bottom
    }
}