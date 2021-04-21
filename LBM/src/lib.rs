extern crate libc;

use libc::size_t;
use std::slice;

use arrayfire::*;
use af_opencl_interop as afcl;
use ocl::{core, flags, Event};
use ocl::enums::{ArgVal};
use std::ffi::CString;
use libc::c_char;

mod lbm;
use lbm::lbm_d2q9;

#[repr(C)]
struct Sim {
    pub results_ptr: *const u8
}

impl Sim {
    #[no_mangle]
    pub unsafe extern "C" fn init_array() -> *const u8 {
        set_device(0);
        set_backend(Backend::CUDA);

        let dims = Dim4::new(&[128, 128, 4, 1]);
        let a = randu::<u8>(dims);
        let mut results = vec!(0u8; a.elements());
        a.host(&mut results);

        results.as_ptr()
        // match CString::new(results) {
        //     Ok(c_str_4) => println!("Got a C string: {:p}", c_str_4.as_ptr()),
        //     Err(e) => println!("Error getting a C string: {}", e),
        // }  
    }
}

#[no_mangle]
pub unsafe extern "C" fn init_array() -> *const u8 {
    set_device(0);
    set_backend(Backend::CUDA);

    let dims = Dim4::new(&[128, 128, 4, 1]);
    let a = randu::<u8>(dims);
    let mut results = vec!(0u8; a.elements());
    a.host(&mut results);

    results.as_ptr()
    // match CString::new(results) {
    //     Ok(c_str_4) => println!("Got a C string: {:p}", c_str_4.as_ptr()),
    //     Err(e) => println!("Error getting a C string: {}", e),
    // }  
}

#[no_mangle]
pub unsafe extern "C" fn init_array_ptr() -> *const c_char {
    set_device(0);
    set_backend(Backend::CUDA);

    let dims = Dim4::new(&[64, 64, 4, 1]);
    let a = randu::<u8>(dims);
    let mut results = vec!(0u8; a.elements());
    a.host(&mut results);

    let host = CString::new(results).expect("CString::new failed");
    host.as_ptr()
    // match CString::new(results) {
    //     Ok(c_str_4) => println!("Got a C string: {:p}", c_str_4.as_ptr()),
    //     Err(e) => println!("Error getting a C string: {}", e),
    // }  
}

#[no_mangle]
pub extern "C" fn computation_slow() -> *mut f32 {
    set_device(0);
    set_backend(Backend::CUDA);

    let dims = Dim4::new(&[10 * 10, 1, 1, 1]);
    let a = constant::<f32>(0.0 as f32, dims);
    let mut results = vec!(0.0f32; a.elements());
    let ptr = results.as_mut_ptr();

    // std::thread::spawn(move || {
        set_device(0);
        set_backend(Backend::CUDA);

        lbm_d2q9(&mut results);
    // });

    ptr
}

// #[no_mangle]
// pub unsafe extern "C" fn cleanup(ptr: *mut f32, len: size_t) {
//     drop(Vec::from_raw_parts(ptr, len as usize, len as usize));
// }

unsafe fn get_ptr(a: Array<f32>) -> *mut std::ffi::c_void {
    a.device_ptr()
}
