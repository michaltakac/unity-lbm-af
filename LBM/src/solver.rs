use crate::boundary;
use crate::grid;
use crate::traits::{Distribution, DistributionStorage, Physics};
use crate::FloatNum;
use arrayfire::*;
// use time;

/// Lattice-Boltzmann Solver state
pub struct Solver<P: Physics> {
    grid: grid::StructuredRectangular,
    pub bcs: boundary::Handler,
    physics: P,
    f: Array<FloatNum>, // Distribution functions
    f_hlp: Array<FloatNum>,
}

impl<P: Physics> Solver<P> {
    /// Create a new solver from a `grid` and `physics`.
    pub fn new(grid: grid::StructuredRectangular, bcs: boundary::Handler, physics: P) -> Solver<P> {
        Solver {
            grid,
            bcs,
            physics,
            f: constant(
                0.0 as FloatNum,
                dim4!(grid.size() * P::Distribution::size()),
            ),
            f_hlp: constant(
                0.0 as FloatNum,
                dim4!(grid.size() * P::Distribution::size()),
            ),
        }
    }

    /// Initialize distributions
    pub fn initialize(&mut self) {
        unimplemented!();
    }

    /// Is the cell `c` par of a solid boundary?
    fn solid_boundary(&self, c: grid::Idx) -> bool {
        self.bcs.solid_boundary(self.grid.x(c))
    }

    /// Streaming step
    fn streaming(&mut self) {
        let mut f_hlp = std::mem::replace(&mut self.f_hlp, Default::default());
        let f_streamed = view!(f_hlp[P::Distribution::neighbors_index()]);
        self.f_hlp = f_hlp;
    }

    /// Collision step
    fn collision(&mut self) {
        let mut f = ::std::mem::replace(&mut self.f, Default::default());
        let f_streamed = view!(f[P::Distribution::neighbors_index()]);
        self.f = f_streamed;
    }

    /// Executes `n_it` iterations writing output every `n_out` iterations.
    pub fn run(&mut self, n_it: usize, n_out: usize) {
        let mut n_it = n_it;
        assert!(n_it > 0);
        let mut iter = 0;
        use time::Duration;

        loop {
            let write_output = n_out > 0 && iter % n_out == 0;
            let d = Duration::span(|| {
                let d = Duration::span(|| self.streaming());
                if write_output {
                    self.substep("propagation", d);
                }

                let d = Duration::span(|| self.collision());
                if write_output {
                    self.substep("collision", d);
                }

                let d = Duration::span(|| self.apply_boundary_conditions());
                if write_output {
                    self.substep("bcs", d);
                }

                n_it -= 1;
                if write_output {
                    let d = Duration::span(|| self.visualize(iter));
                    self.substep("vtk", d);
                }
            });
            if write_output {
                self.step(iter, d);
            }
            if n_it == 0 {
                break;
            }
            iter += 1;
        }
    }

    /// Prints line info of a whole iteration step
    fn step(&self, n_it: usize, duration: time::Duration) {
        let integral = self.integral();
        println!(
            "#{} | integral: {} | duration: {} ms",
            n_it,
            integral,
            duration.num_milliseconds()
        );
    }
    /// Prints line info of an iteration sub-step
    fn substep(&self, name: &str, duration: time::Duration) {
        let res = self.integral();
        println!(
            "# [{}] | integral: {} | duration: {} \u{03BC}s",
            name,
            res,
            duration.num_microseconds().unwrap()
        );
    }
    /// Writes the solution to a VTK file.
    fn visualize(&self, iter: usize) {
        unimplemented!();
    }
}
