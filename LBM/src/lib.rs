mod lbm;
use lbm::Sim;
use std::ptr;

#[macro_use]
mod debug;

pub type FloatNum = f32;

#[no_mangle]
pub extern "C" fn init_sim(
    ptr: *mut *mut Sim,
    data_ptr: *mut *mut u8,
    width: u32,
    height: u32,
    initial_density: FloatNum,
    initial_ux: FloatNum,
    omega: FloatNum,
    obstacle_x: u32,
    obstacle_y: u32,
    obstacle_r: u32,
) -> bool {
    match Sim::init_sim_channel(
        width.into(),
        height.into(),
        initial_density,
        initial_ux,
        omega,
        obstacle_x.into(),
        obstacle_y.into(),
        obstacle_r.into(),
    ) {
        Ok(sim) => {
            unsafe {
                *ptr = sim.to_ptr();
                *data_ptr = Sim::from_ptr(*ptr).results_ptr();
            }

            true
        }
        Err(message) => {
            unsafe {
                *ptr = ptr::null_mut();
            }
            debug!("Error when initializing LBM simulation: {:}", message);
            false
        }
    }
}

#[no_mangle]
pub extern "C" fn simulate(ptr: *mut Sim, inflow_density: FloatNum, inflow_ux: FloatNum, omega: FloatNum) {
    if !ptr.is_null() {
        Sim::from_ptr(ptr).simulate(inflow_density, inflow_ux, omega);
    }
}

#[no_mangle]
pub extern "C" fn get_sim_data(ptr: *mut Sim) {
    if !ptr.is_null() {
        Sim::from_ptr(ptr).copy_colors_host();
    }
}
