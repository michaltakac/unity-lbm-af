pub mod lbm;
pub mod grid;
pub mod boundary;
pub mod distribution;
pub mod geometry;
pub mod physics;
pub mod traits;
pub mod streaming;
pub mod solver;
pub mod lib;

use arrayfire::*;
use self::solver::Solver;
use self::lib::FloatNum;

// Configure the numerical method
type Distribution = distribution::D2Q9;
type Collision = physics::ns::SingleRelaxationTime;
type Physics = physics::NavierStokes<Distribution, Collision>;

fn main() {
    // Initialize the grid, physical parameters, and solver:
    let grid = grid::StructuredRectangular::<Distribution>::new(300, 150, 0);

    // Add initial static boundary conditions
    let bcs = boundary::Handler::<Distribution>::new(grid);
    {
        // Cylinder:
        {
            let cyl = Box::new(boundary::Condition::new(
                boundary::Type::BounceBack,
                geometry::Circle::new(grid.x, grid.y),
            ));
            bcs.add("cylinder", cyl);
        }
        // Bottom channel wall:
        {
            let bottom_wall = Box::new(boundary::Condition::new(
                boundary::Type::BounceBack,
                geometry::Plane::new((0, 1), (0, 0)),
            ));
            bcs.add("bottom_wall", bottom_wall);
        }
        // Top channel wall:
        {
            let top_wall = Box::new(boundary::Condition::new(
                boundary::Type::BounceBack,
                geometry::Plane::new((0, -1), (0, grid.y - 1)),
            ));
            bcs.add("top_wall", top_wall);
        }
    }

    // Initialize physics
    let physics: Physics = Physics::new(0.1, 0.015, Collision { omega: 1.85, re: 220.0 });

    // Initialize physical boundary conditions
    {
        // Periodic forced inflow:
        {
            let bc = Box::new(boundary::Condition::new(
                boundary::Type::Inflow(
                    physics.inflow_density,
                    physics.inflow_accel,
                ),
                geometry::Plane::new((1, 0), (0, 0)),
            ));
            bcs.add("left_wall_inflow", bc);
        }
    }

    let mut s = Solver::new(grid, bcs, physics);

    // Initialize distribution functions
    s.initialize(|_| {
        let mut ns = DistributionStorage::<Dist>::default();
        for n in Dist::all() {
            ns.as_mut()[n.value()] = physics.inflow_density * n.constant();
        }
        ns
    });

    s.run(10001, 500);
}