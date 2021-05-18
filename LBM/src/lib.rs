#[macro_use]
mod debug;
pub mod lbm;
pub mod grid;
pub mod boundary;
pub mod geometry;
pub mod physics;
pub mod traits;
pub mod streaming;
pub mod distribution;
pub mod solver;
use lbm::Sim;
use std::ptr;

pub type FloatNum = f32;


#[no_mangle]
pub extern "C" fn init_sim(ptr: *mut *mut Sim, data_ptr: *mut *mut u8) -> bool {
    match Sim::init_sim_channel() {
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
pub extern "C" fn simulate(ptr: *mut Sim) {
    if !ptr.is_null() {
        Sim::from_ptr(ptr).simulate();
    }
}

#[no_mangle]
pub extern "C" fn get_sim_data(ptr: *mut Sim) {
    if !ptr.is_null() {
        Sim::from_ptr(ptr).copy_colors_host();
    }
}
