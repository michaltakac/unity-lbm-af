use crate::FloatNum;
use arrayfire::*;

fn normalize(a: &Array<FloatNum>) -> Array<FloatNum> {
    let min = min_all(a).0;
    let max = max_all(a).0;
    (a - min) / (max - min) as FloatNum
}

fn stream_2d(f: &Array<FloatNum>) -> Array<FloatNum> {
    let mut pdf = f.clone();
    eval!(pdf[1:1:0, 1:1:0, 1:1:1] = shift(&view!(f[1:1:0, 1:1:0, 1:1:1]), &[1, 0, 0, 0]));
    eval!(pdf[1:1:0, 1:1:0, 2:2:1] = shift(&view!(f[1:1:0, 1:1:0, 2:2:1]), &[0, 1, 0, 0]));
    eval!(pdf[1:1:0, 1:1:0, 3:3:1] = shift(&view!(f[1:1:0, 1:1:0, 3:3:1]), &[-1, 0, 0, 0]));
    eval!(pdf[1:1:0, 1:1:0, 4:4:1] = shift(&view!(f[1:1:0, 1:1:0, 4:4:1]), &[0, -1, 0, 0]));
    eval!(pdf[1:1:0, 1:1:0, 5:5:1] = shift(&view!(f[1:1:0, 1:1:0, 5:5:1]), &[1, 1, 0, 0]));
    eval!(pdf[1:1:0, 1:1:0, 6:6:1] = shift(&view!(f[1:1:0, 1:1:0, 6:6:1]), &[-1, 1, 0, 0]));
    eval!(pdf[1:1:0, 1:1:0, 7:7:1] = shift(&view!(f[1:1:0, 1:1:0, 7:7:1]), &[-1, -1, 0, 0]));
    eval!(pdf[1:1:0, 1:1:0, 8:8:1] = shift(&view!(f[1:1:0, 1:1:0, 8:8:1]), &[1, -1, 0, 0]));
    pdf
}

#[repr(C)]
pub struct Sim {
    sim_type: String,
    results: Vec<u8>,
    results_ptr: *mut u8,
    total_nodes: u64,
    nx: u64,
    ny: u64,
    obstacle_x: u64,
    obstacle_y: u64,
    obstacle_r: u64,
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
}

impl<'a> Sim {
    pub fn to_ptr(self) -> *mut Sim {
        let sim_boxed = Box::new(self);
        Box::into_raw(sim_boxed)
    }

    pub fn from_ptr(ptr: *mut Sim) -> &'a mut Sim {
        unsafe { &mut *ptr }
    }

    pub fn results_ptr(&self) -> *mut u8 {
        self.results_ptr
    }

    pub fn init_sim_channel(
        nx: u64,
        ny: u64,
        initial_density: FloatNum,
        initial_ux: FloatNum,
        omega: FloatNum,
        obstacle_x: u64,
        obstacle_y: u64,
        obstacle_r: u64,
    ) -> Result<Sim, String> {
        set_device(0);
        set_backend(Backend::OPENCL);

        let total_nodes = nx * ny;

        let t1: FloatNum = 4. / 9.;
        let t2: FloatNum = 1. / 9.;
        let t3: FloatNum = 1. / 36.;

        let x: Array<FloatNum> = tile(&range(dim4!(nx), 0), dim4!(1, ny));
        let y: Array<FloatNum> = tile(&range(dim4!(1, ny), 1), dim4!(nx, 1));

        let dims = dim4!(nx, ny);
        let dims_rgba = dim4!(nx, ny, 4);

        //  c6  c2   c5
        //    \  |  /
        //  c3 -c0 - c1
        //    /  |  \
        //  c7  c4   c8
        // Discrete velocities
        let ex = Array::<FloatNum>::new(&[0., 1., 0., -1., 0., 1., -1., -1., 1.], dim4!(9));
        let ey = Array::<FloatNum>::new(&[0., 0., 1., 0., -1., 1., 1., -1., -1.], dim4!(9));

        // weights
        let w = Array::new(&[t1, t2, t2, t2, t2, t3, t3, t3, t3], dim4!(9));

        let ci: Array<u64> = (range::<u64>(dim4!(1, 8), 1) + 1) * total_nodes;
        let nbidx = Array::new(&[2, 3, 0, 1, 6, 7, 4, 5], dim4!(8));
        let span = seq!();
        let nbi: Array<u64> = view!(ci[span, nbidx]);

        let main_index = moddims(&range(dim4!(total_nodes * 9), 0), dim4!(nx, ny, 9));
        let nb_index = flat(&stream_2d(&main_index));

        // Flow around obstacle
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

        // matrix offset of each Occupied Node
        let on = locate(&bound);

        // Bounceback indexes
        let to_reflect = flat(&tile(&on, dim4!(ci.elements() as u64)))
            + flat(&tile(&ci, dim4!(on.elements() as u64)));
        let reflected = flat(&tile(&on, dim4!(nbi.elements() as u64)))
            + flat(&tile(&nbi, dim4!(on.elements() as u64)));

        let mut density = constant::<FloatNum>(initial_density, dims);
        let mut ux = constant::<FloatNum>(initial_ux, dims);
        let mut uy = constant::<FloatNum>(0.0, dims);

        let zeroed_on = constant::<FloatNum>(0.0, on.dims());

        eval!(ux[on] = zeroed_on);
        eval!(uy[on] = zeroed_on);
        eval!(density[on] = zeroed_on);

        // Start in equilibrium state
        let u_sq: Array<FloatNum> =
            flat(&(pow(&ux, &(2.0 as FloatNum), false) + pow(&uy, &(2.0 as FloatNum), false)));
        let eu: Array<FloatNum> = flat(
            &(&mul(&transpose(&ex, false), &flat(&ux), true)
                + &mul(&transpose(&ey, false), &flat(&uy), true)),
        );
        let f: Array<FloatNum> = flat(&mul(&transpose(&w, false), &flat(&density), true))
            * ((1.0 as FloatNum)
                + (3.0 as FloatNum) * &eu
                + (4.5 as FloatNum) * (&pow(&eu, &(2.0 as FloatNum), false))
                - (1.5 as FloatNum) * (&tile(&flat(&u_sq), dim4!(9))));

        let colors = constant::<u8>(0u8, dims_rgba);
        let mut results = vec![0u8; colors.elements()];
        let results_ptr = results.as_mut_ptr();

        sync(0);
        println!("Simulation started...");

        Ok(Sim {
            sim_type: "channel".to_owned(),
            colors,
            dims,
            dims_rgba,
            results,
            results_ptr,
            omega,
            obstacle_x,
            obstacle_y,
            obstacle_r,
            f,
            on,
            zeroed_on,
            bound,
            nb_index,
            to_reflect,
            reflected,
            density,
            ux,
            uy,
            ex,
            ey,
            eu,
            u_sq,
            total_nodes,
            nx,
            ny,
            w,
        })
    }

    pub fn simulate(&mut self, inflow_density: FloatNum, inflow_ux: FloatNum, omega: FloatNum) {
        // Streaming by reading from neighbors (with pre-built index) - pull scheme
        let mut f = self.f.clone();
        let f_streamed = view!(f[self.nb_index]);

        let bouncedback = view!(f_streamed[self.to_reflect]); // Densities bouncing back at next timestep

        let f_2d = moddims(&f_streamed, dim4!(self.total_nodes, 9));

        // Compute macroscopic variables
        let rho = sum(&f_2d, 1);
        let mut density = moddims(&rho, self.dims);

        let fex = mul(&transpose(&self.ex, false), &f_2d, true);
        let fey = mul(&transpose(&self.ey, false), &f_2d, true);

        let mut ux = moddims(&(sum(&fex, 1) / &rho), self.dims);
        let mut uy = moddims(&(sum(&fey, 1) / &rho), self.dims);

        eval!(density[self.on] = self.zeroed_on);

        // inlet speed
        let u_max_af = constant::<FloatNum>(inflow_ux, dim4!(self.ny));
        eval!(ux[0:0:1, 1:1:0] = u_max_af);

        set_row(&mut density, &constant::<FloatNum>(inflow_density, dim4!(self.ny)), 0);
        set_row(
            &mut density,
            &constant::<FloatNum>(inflow_density, dim4!(self.ny)),
            self.nx as i64 - 1,
        );

        eval!(ux[self.on] = self.zeroed_on);
        eval!(uy[self.on] = self.zeroed_on);

        // Collision
        let u_sq =
            flat(&(pow(&ux, &(2.0 as FloatNum), false) + pow(&uy, &(2.0 as FloatNum), false)));
        let eu = flat(
            &(&mul(&transpose(&self.ex, false), &flat(&ux), true)
                + &mul(&transpose(&self.ey, false), &flat(&uy), true)),
        );
        let feq = flat(&mul(&transpose(&self.w, false), &flat(&density), true))
            * ((1.0 as FloatNum)
                + (3.0 as FloatNum) * &eu
                + (4.5 as FloatNum) * (&pow(&eu, &(2.0 as FloatNum), false))
                - (1.5 as FloatNum) * (&tile(&flat(&u_sq), dim4!(9))));

        f = omega * &feq + (1.0 - omega) * &f_streamed;

        eval!(f[self.reflected] = bouncedback);
        self.f = f;

        // Results
        let mut results = moddims(&sqrt(&u_sq), self.dims);
        eval!(results[self.on] = constant::<FloatNum>(FloatNum::NAN, self.on.dims()));
        results = normalize(&results);
        // Colormap for Unity's Texture2D (RGBA32)
        let r = flat(&((1.5f32 - abs(&(1.0f32 - 4.0f32 * (&results - 0.5f32)))) * 255));
        let g = flat(&((1.5f32 - abs(&(1.0f32 - 4.0f32 * (&results - 0.25f32)))) * 255));
        let b = flat(&((1.5f32 - abs(&(1.0f32 - 4.0f32 * &results))) * 255));
        let a = flat(&constant::<f32>(1.0 as FloatNum, self.dims));
        self.colors = flat(&transpose(&join_many(1, vec![&r, &g, &b, &a]), false)).cast::<u8>();

        sync(0);
    }

    pub fn copy_colors_host(&mut self) {
        self.colors.host(self.results.as_mut_slice());
    }
}
