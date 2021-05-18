use std;
use arrayfire::*;
use crate::FloatNum;
use crate::grid;
use crate::distribution;

/// Navier-Stokes distributions:
pub trait Distribution {
    #[inline(always)]
    fn density(f: &Array<FloatNum>) -> FloatNum {
        moddims(&f, dim4!(Self::size(), Self::directions()));
    }

    #[inline(always)]
    fn pressure(f: &Array<FloatNum>) -> FloatNum {
        Self::density(f) * Self::c_squ()
    }

    #[inline(always)]
    fn velocity(f: F, d: usize) -> FloatNum {
        let mut tmp = 0.;
        for n in Self::all() {
            tmp += n.direction().FloatNum_array()[d] * f(n);
        }
        tmp / Self::density(f)
    }

    #[inline(always)]
    fn velocities<F: Fn(Self) -> FloatNum>(f: F) -> [FloatNum; 2] {
        [Self::velocity(&f, 0), Self::velocity(&f, 1)]
    }
}

impl Distribution for distribution::D2Q9 {}
// impl Distribution for distribution::D3Q27 {}

/// Single relaxation time (SRT) algorithm
#[derive(Copy, Clone)]
pub struct SingleRelaxationTime {
    pub omega: FloatNum,
}

impl<D: Distribution> ::Collision<D> for SingleRelaxationTime {
    #[inline(always)]
    fn collision<H>(&self, f_hlp: &H) -> D::Storage {
        // local density and vel components:
        let f_2d = moddims(&f_hlp, dim4!(self.total_nodes, 9));

        // Compute macroscopic variables
        let rho = sum(&f_2d, 1);
        let mut density = moddims(&rho, self.dims);

        let fex = mul(&transpose(&self.ex, false), &f_2d, true);
        let fey = mul(&transpose(&self.ey, false), &f_2d, true);

        let mut ux = moddims(&(sum(&fex, 1) / &rho), self.dims);
        let mut uy = moddims(&(sum(&fey, 1) / &rho), self.dims);
        let dloc = D::density(&f_h);
        let [u_x, u_y] = D::velocities(&f_h);

        // n- velocity compnents (n = grid node connection vectors)
        // TODO: switch to 3 speeds only
        let mut u_n_ = D::Storage::default();
        {
            let u_n = u_n_.as_mut();
            for n in D::all() {
                let v = n.direction().num_array();
                let n = n.value();
                u_n[n] = v[0] * u_x + v[1] * u_y;
            }

            // equilibrium densities:
            let f0 = 2. * D::c_squ() * D::c_squ();
            let f1 = 2. * D::c_squ();
            let u_squ = u_x.powf(2.) + u_y.powf(2.); // square velocity
            let f2 = u_squ / f1;

            let mut n_equ_ = D::Storage::default();
            let n_equ = n_equ_.as_mut();

            // zero-th velocity density
            n_equ[0] = D::center().constant() * dloc * (1. - f2);

            for n in D::direct() {
                let f3 = n.constant() * dloc;
                let n = n.value();
                n_equ[n] =
                    f3 * (1. + u_n[n] / D::c_squ() + u_n[n].powf(2.) / f0 - f2);
            }
            for n in D::diagonal() {
                let f4 = n.constant() * dloc;
                let n = n.value();
                n_equ[n] =
                    f4 * (1. + u_n[n] / D::c_squ() + u_n[n].powf(2.) / f0 - f2);
            }

            // relaxation step:
            for n in D::all() {
                u_n[n.value()] =
                    f_h(n) + self.omega * (n_equ[n.value()] - f_h(n));
            }

            // Collision
            let u_sq = flat(&(pow(&ux, &(2.0 as FloatNum), false) + pow(&uy, &(2.0 as FloatNum), false)));
            let eu = flat(
                &(&mul(&transpose(&self.ex, false), &flat(&ux), true)
                    + &mul(&transpose(&self.ey, false), &flat(&uy), true)),
            );
            let feq = flat(&mul(&transpose(&self.w, false), &flat(&density), true))
                * ((1.0 as FloatNum)
                    + (3.0 as FloatNum) * &eu
                    + (4.5 as FloatNum) * (&pow(&eu, &(2.0 as FloatNum), false))
                    - (1.5 as FloatNum) * (&tile(&flat(&u_sq), dim4!(9))));

            // Relaxation step
            f = self.omega * &feq + (1.0 - self.omega) * &f_streamed;

            eval!(f[self.reflected] = bouncedback);
            self.f = f;
        }
        u_n_
    }
}

#[derive(Copy, Clone)]
pub struct NavierStokes<D: Distribution, C: ::Collision<D>> {
    pub inflow_density: FloatNum,
    pub inflow_accel: FloatNum,
    collision: C,
    __dist: std::marker::PhantomData<D>,
}

impl<D: Distribution, C: ::Collision<D>> NavierStokes<D, C> {
    pub fn new(density: FloatNum, accel: FloatNum, col: C) -> Self {
        Self {
            inflow_density: density,
            inflow_accel: accel,
            collision: col,
            __dist: std::marker::PhantomData {},
        }
    }
    #[inline(always)]
    pub fn pressure<F: Fn(D) -> FloatNum>(&self, solid: bool, f: F) -> FloatNum {
        if solid {
            self.inflow_density * D::c_squ()
        } else {
            D::pressure(f)
        }
    }
    #[inline(always)]
    pub fn velocities<F: Fn(D) -> FloatNum>(solid: bool, f: F) -> [FloatNum; 2] {
        if solid {
            [0., 0.]
        } else {
            D::velocities(f)
        }
    }
}

impl<D: Distribution, C: ::Collision<D>> crate::traits::Physics
    for NavierStokes<D, C> {
    type Distribution = D;
    #[inline(always)]
    fn collision<H, IH>(&self, f_hlp: &H, idx_h: IH) -> D::Storage
    where
        IH: Fn(&H, D) -> FloatNum,
    {
        self.collision.collision(f_hlp, idx_h)
    }
    #[inline(always)]
    fn integral(f: Array<FloatNum>) {
        D::density(f)
    }

    fn visualize(&self) {
        unimplemented!();
    }
}