use arrayfire::*;
use crate::FloatNum;

fn normalize(a: &Array<FloatNum>) -> Array<FloatNum> {
    let min = min_all(a).0;
    let max = max_all(a).0;
    (a-min)/(max-min) as FloatNum
}




pub enum SimType {
    LID,
    CHANNEL,
}

pub trait Simulation {
    fn set_omega(&mut self);
    fn set_initial_conditions(&mut self);
}

#[repr(C)]
pub struct ChannelSimulation {
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

#[repr(C)]
pub struct LidPhysics {
    ux_lid: Array<FloatNum>,
    uy_lid: Array<FloatNum>,
}

#[repr(C)]
pub struct Physics2D {
    top_boundary_u: Array<FloatNum>,
    bottom_boundary_u: Array<FloatNum>,
    left_boundary_u: Array<FloatNum>,
    right_boundary_u: Array<FloatNum>,
}


#[repr(C)]
pub struct Sim {
    sim_type: String,
    results: Vec<u8>,
    results_ptr: *mut u8,
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

    pub fn init_sim_2d(nx: u64, ny: u64, platform: Option<Backend>, physics: Physics) -> Result<Sim, String> {
        // Not supporting multi-GPU, for now use first device that is available
        set_device(0);

        // Make default backend the OpenCL (cross-platform)
        let p = match platform {
            Some(p) => p,
            None => Backend::OPENCL,
        };
        set_backend(p);

        // Initialize results that are sent to Unity
        let dims_rgba = dim4!(nx * ny * 4);
        let colors = constant::<u8>(0u8, dims_rgba);
        let mut results = vec![0u8; colors.elements()];
        let results_ptr = results.as_mut_ptr();

        let total_nodes = nx * ny;

        // Physical parameters based on simulation type

        Ok(Sim {
            results,
            results_ptr,
            nx,
            ny,

        })
    }

    pub fn init_sim_lid() -> Result<Sim, String> {
        set_device(0);
        set_backend(Backend::OPENCL);
        // Grid length, number and spacing
        let nx: u64 = 128;
        let ny: u64 = 128;

        let total_nodes = nx * ny;

        // Physical parameters.
        let ux_lid: FloatNum = 0.05; // horizontal lid velocity
        let uy_lid: FloatNum = 0.0; // vertical lid velocity
        let rho0: FloatNum = 1.0;

        // Reynolds number
        let re: FloatNum = 100.0;
        // Kinematic viscosity
        let nu: FloatNum = ux_lid * 2.0 * nx as FloatNum / re;
        // Relaxation time
        let tau: FloatNum = (3.0 as FloatNum) * nu + (0.5 as FloatNum);
        // Relaxation parameter
        let omega: FloatNum = (1.0 as FloatNum) / tau;

        let t1: FloatNum = 4. / 9.;
        let t2: FloatNum = 1. / 9.;
        let t3: FloatNum = 1. / 36.;

        let dims = dim4!(nx, ny);
        let dims_rgba = dim4!(nx, ny, 4);

        let ux_lid_af = constant::<FloatNum>(ux_lid, dims);
        let uy_lid_af = constant::<FloatNum>(uy_lid, dims);

        let lid = seq!(1, nx as i32 - 2, 1);
        // TODO: move and recreate when needed?
        // let end_y = seq!(nx as i32 - 1, ny as i32 - 1, 1);

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
        let nb_index = flat(&stream(&main_index));

        // Open lid
        let mut bound = constant::<FloatNum>(1.0, dims);
        let zeros = constant::<FloatNum>(0.0, dims);
        let all_except_top_lid = seq!(1, ny as i32 - 1, 1);
        assign_seq(
            &mut bound,
            &[lid, all_except_top_lid],
            &index(&zeros, &[lid, all_except_top_lid]),
        );

        // matrix offset of each Occupied Node
        let on = locate(&bound);

        // Bounceback indexes
        let to_reflect = flat(&tile(&on, dim4!(ci.elements() as u64)))
            + flat(&tile(&ci, dim4!(on.elements() as u64)));
        let reflected = flat(&tile(&on, dim4!(nbi.elements() as u64)))
            + flat(&tile(&nbi, dim4!(on.elements() as u64)));

        let density = constant::<FloatNum>(rho0, dims);
        let ux = constant::<FloatNum>(0.0, dims);
        let uy = constant::<FloatNum>(0.0, dims);

        let zeroed_on = constant::<FloatNum>(0.0, on.dims());

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

        Ok(Sim {
            sim_type: "lid".to_owned(),
            colors,
            dims,
            dims_rgba,
            results,
            results_ptr,
            omega,
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
            ux_lid_af,
            uy_lid_af
        })
    }

    pub fn init_sim_channel() -> Result<Sim, String> {
        set_device(0);
        set_backend(Backend::OPENCL);
        // Grid length, number and spacing
        let nx: u64 = 300;
        let ny: u64 = 100;

        let total_nodes = nx * ny;

        // Physical parameters.
        let rho0: FloatNum = 1.0;

        let obstacle_x: u64 = nx / 5 + 1; // x location of the cylinder
        let obstacle_y: u64 = ny / 2 + ny / 30; // y location of the cylinder
        let obstacle_r: u64 = ny / 10 + 1; // radius of the cylinder

        // Reynolds number
        let re: FloatNum = 220.0;
        // Lattice speed
        let u_max: FloatNum = 0.1;
        let u_max_af = constant::<FloatNum>(u_max, dim4!(ny));
        // Kinematic viscosity
        let nu: FloatNum = u_max * 2.0 * obstacle_r as FloatNum / re;
        // Relaxation time
        let tau: FloatNum = (3.0 as FloatNum) * nu + (0.5 as FloatNum);
        // Relaxation parameter
        let omega: FloatNum = (1.0 as FloatNum) / tau;

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
        let nb_index = flat(&stream(&main_index));

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

        let mut density = constant::<FloatNum>(rho0, dims);
        let mut ux = constant::<FloatNum>(u_max, dims);
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
            ux_lid_af: Array::new_empty(dim4!(1)),
            uy_lid_af: Array::new_empty(dim4!(1))
        })
    }

    pub fn simulate(&mut self) {
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

        if self.sim_type == "lid".to_owned() {
            // Macroscopic (Dirichlet) boundary conditions
            let lid = seq!(1, self.nx as i32 - 2, 1);
            let end_y = seq!(self.nx as i32 - 1, self.ny as i32 - 1, 1);
            let ux_lid_af = self.ux_lid_af.clone();
            let uy_lid_af = self.uy_lid_af.clone();
            eval!(ux[lid, end_y] = view!(ux_lid_af[lid, end_y]));
            eval!(uy[lid, end_y] = view!(uy_lid_af[lid, end_y]));
        } else if self.sim_type == "channel".to_owned() {
            // inlet speed
            let u_max: FloatNum = 0.1;
            let u_max_af = constant::<FloatNum>(u_max, dim4!(self.ny));
            eval!(ux[0:0:1, 1:1:0] = u_max_af);

            set_row(&mut density, &constant::<FloatNum>(1.0, dim4!(self.ny)), 0);
            set_row(
                &mut density,
                &constant::<FloatNum>(1.0, dim4!(self.ny)),
                self.nx as i64 - 1,
            );
        }

        eval!(ux[self.on] = self.zeroed_on);
        eval!(uy[self.on] = self.zeroed_on);

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

        f = self.omega * &feq + (1.0 - self.omega) * &f_streamed;

        eval!(f[self.reflected] = bouncedback);
        self.f = f;

        // Results
        let mut results = moddims(&sqrt(&u_sq), self.dims);
        eval!(results[self.on] = constant::<FloatNum>(FloatNum::NAN, self.on.dims()));
        results = normalize(&results);
        // Colormap for Unity's Texture2D (RGBA32)
        let r = flat(&((1.5f32-abs(&(1.0f32-4.0f32*(&results-0.5f32)))) * 255));
        let g = flat(&((1.5f32-abs(&(1.0f32-4.0f32*(&results-0.25f32)))) * 255));
        let b = flat(&((1.5f32-abs(&(1.0f32-4.0f32*&results))) * 255));
        let a = flat(&constant::<f32>(1.0 as FloatNum, self.dims));
        self.colors = flat(&transpose(&join_many(1, vec![&r, &g, &b, &a]), false)).cast::<u8>();

        sync(0);
    }

    pub fn copy_colors_host(&mut self) {
        self.colors.host(self.results.as_mut_slice());
    }
}