extern crate libc;

use libc::size_t;
use std::slice;

use arrayfire::*;
use af_opencl_interop as afcl;
use ocl::{core, flags, Event};
use ocl::enums::{ArgVal};
use ocl::builders::ContextProperties;
use std::ffi::CString;
use libc::c_char;

mod lbm;
use lbm::lbm_d2q9;

#[repr(C)]
struct Sim {
    pub results_ptr: *const u8
}

// impl Sim {
//     #[no_mangle]
//     pub unsafe extern "C" fn init_array() -> *const u8 {
//         set_device(0);
//         set_backend(Backend::CUDA);

//         let dims = Dim4::new(&[128, 128, 4, 1]);
//         let a = randu::<u8>(dims);
//         let mut results = vec!(0u8; a.elements());
//         a.host(&mut results);

//         results.as_ptr()
//         // match CString::new(results) {
//         //     Ok(c_str_4) => println!("Got a C string: {:p}", c_str_4.as_ptr()),
//         //     Err(e) => println!("Error getting a C string: {}", e),
//         // }  
//     }
// }

#[no_mangle]
pub unsafe extern "C" fn init_array() -> *const u8 {
    set_device(0);
    set_backend(Backend::OPENCL);

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
    set_backend(Backend::OPENCL);

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
pub extern "C" fn init_array_opencl(ptr: *const u8) {
    set_device(0);
    set_backend(Backend::OPENCL);

    // Choose platform & device(s) to use. Create a context, queue,
    let platform_id = ocl_core::default_platform().unwrap();
    let device_ids = ocl_core::get_device_ids(&platform_id, None, None).unwrap();
    let device_id = device_ids[0];
    let context_properties = ContextProperties::new().platform(platform_id);
    let context =
        ocl_core::create_context(Some(&context_properties), &[device_id], None, None).unwrap();
    let queue = ocl_core::create_command_queue(&context, &device_id, None).unwrap();
    let dims = [8, 1, 1];

    // Create a `Buffer`:
    let mut vec = vec![0.0f32; dims[0]];
    let buffer = unsafe {
        ocl_core::create_buffer(
            &context,
            ocl_core::MEM_READ_WRITE | ocl_core::MEM_COPY_HOST_PTR,
            dims[0],
            Some(&vec),
        )
        .unwrap()
    };
    ocl_core::finish(&queue).unwrap(); //sync up before switching to arrayfire

    // Add custom device, context and associated queue to ArrayFire
    afcl::add_device_context(device_id.as_raw(), context.as_ptr(), queue.as_ptr());
    afcl::set_device_context(device_id.as_raw(), context.as_ptr());
    info();

    let mut af_buffer = Array::new_from_device_ptr(
        buffer.as_ptr() as *mut f32,
        Dim4::new(&[dims[0] as u64, 1, 1, 1]),
    );

    af_print!("GPU Buffer before modification:", af_buffer);

    af_buffer = af_buffer + 10f32;

    sync(get_device());
    unsafe {
        let ptr = af_buffer.device_ptr();
        let obuf = ocl_core::Mem::from_raw_copied_ptr(ptr);

        // Read results from the device into a vector:
        ocl_core::enqueue_read_buffer(
            &queue,
            &obuf,
            true,
            0,
            &mut vec,
            None::<Event>,
            None::<&mut Event>,
        )
        .unwrap();
    }
    println!("GPU buffer on host after ArrayFire operation: {:?}", vec);

    // Remove device from ArrayFire management towards Application Exit
    set_device(0); // Cannot pop when in Use, hence switch to another device
    afcl::delete_device_context(device_id.as_raw(), context.as_ptr());

    // cl_mem mem = clCreateFromGLTexture(context, CL_MEM_WRITE_ONLY, GL_TEXTURE_2D, 0,texture,NULL);
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
