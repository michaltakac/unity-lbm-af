extern crate libc;
extern crate gl_loader;
extern crate gl;

use libc::size_t;
use std::slice;

use arrayfire::*;
use af_opencl_interop as afcl;
use ocl::{core, flags, Event};
use ocl::enums::{ArgVal};
use ocl::builders::ContextProperties;
use std::ffi::CString;
use libc::c_char;
use std::os::raw::c_void;

use gl::types::*;

mod lbm;
use lbm::lbm_d2q9;

pub type Handle = *mut std::ffi::c_void;

pub trait TextureBuffer {
    unsafe fn ptr(&self) -> *const std::ffi::c_void;
    unsafe fn mut_ptr(&mut self) -> *mut std::ffi::c_void;
    fn row_pitch(&self) -> i32;
}

pub struct TexBuff {
    buffer: Vec<u8>,
    row_ptch: i32,
}

impl TexBuff {
    pub fn new(buffer_size: usize, row_pitch: i32) -> TexBuff {
        let mut buf = Vec::<u8>::with_capacity(buffer_size);
        unsafe {
            buf.set_len(buffer_size);
        }
        TexBuff {
            buffer: buf,
            row_ptch: row_pitch,
        }
    }
}

impl TextureBuffer for TexBuff {
    unsafe fn ptr(&self) -> *const std::ffi::c_void {
        self.buffer.as_ptr() as _
    }

    unsafe fn mut_ptr(&mut self) -> *mut std::ffi::c_void {
        self.buffer.as_mut_ptr() as _
    }

    fn row_pitch(&self) -> i32 {
        self.row_ptch
    }
}

static mut TEXTURE_HANDLE: *mut std::ffi::c_void = std::ptr::null_mut();
static mut TEXTURE_WIDTTH: i32 = 0;
static mut TEXTURE_HEIGHT: i32 = 0;

static mut GRAPHICS: Option<unity_native_plugin::graphics::UnityGraphics> = None;

static mut TIME: f32 = 0.0;

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn SetTimeFromUnity(t: f32) {
    unsafe {
        TIME = t;
    }
}

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
#[allow(non_snake_case)]
pub extern "system" fn SetTextureFromUnity(tex_ptr: *mut std::ffi::c_void, width: i32, height: i32) {
    unsafe {
        TEXTURE_HANDLE = tex_ptr;
        TEXTURE_WIDTTH = width;
        TEXTURE_HEIGHT = height;
    }

    set_device(0);
    set_backend(Backend::OPENCL);

    // Load GL Functions
    gl_loader::init_gl();
    gl::load_with(|symbol| gl_loader::get_proc_address(symbol) as *const _);

	// Update texture data, and free the memory buffer
    let a = randu::<u8>(dim4!(width as u64, height as u64, 4));
    let mut image = vec!(0u8; a.elements());
    a.host(&mut image);
    
    // Choose platform & device(s) to use. Create a context, queue,
    let platform_id = ocl::core::default_platform().unwrap();
    let device_ids = ocl::core::get_device_ids(&platform_id, None, None).unwrap();
    let device_id = device_ids[0];
    let context = ocl_interop::get_context().expect("Cannot find GL's device in CL");
    let queue = ocl::core::create_command_queue(&context, &device_id, None).unwrap();

    // Create a `Buffer`: TODO - it fails!
    // let _cl_buffer = unsafe {
    //     ocl::core::create_from_gl_texture_2d(
    //         context.as_ptr(),
    //         gl::TEXTURE_2D,
    //         0,
    //         tex_ptr as GLuint,
    //         ocl::core::MEM_READ_WRITE,
    //     ).unwrap()
    // };

    // ocl::core::finish(&queue).unwrap(); //sync up before switching to arrayfire
    
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

fn begin_modify_texture(
    handle: *mut c_void,
    texture_width: i32,
    texture_height: i32,
) -> Option<Box<dyn TextureBuffer>> {
    let row_pitch = texture_width * 4;
    Some(Box::new(TexBuff::new(
        (row_pitch * texture_height) as usize,
        row_pitch,
    )))
}

fn end_modify_texture(
    texture_handle: *mut c_void,
    width: i32,
    height: i32,
    buffer: Box<dyn TextureBuffer>,
) {
    unsafe {
        let tex_handle = texture_handle as GLuint;
	    gl::BindTexture(gl::TEXTURE_2D, tex_handle);
	    gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, width, height, gl::RGBA, gl::UNSIGNED_BYTE, buffer.ptr());
    }
}

fn modify_texture_pixels() {
    unsafe {
        let handle = TEXTURE_HANDLE;
        let width = TEXTURE_WIDTTH;
        let height = TEXTURE_HEIGHT;

        if handle.is_null() {
            return;
        }
            if let Some(mut buffer) = begin_modify_texture(handle, width, height) {
                if buffer.ptr().is_null() {
                    return;
                }

                let t = TIME * 4.0;

                let mut dst = buffer.mut_ptr() as *mut u8;
                for y in 0..height {
                    let mut ptr = dst;
                    for x in 0..width {
                        let vv: i32 = ((127.0 + (127.0 * (x as f32 / 7.0 + t).sin()))
                            + (127.0 + (127.0 * (y as f32 / 5.0 - t).sin()))
                            + (127.0 + (127.0 * ((x + y) as f32 / 6.0 - t).sin()))
                            + (127.0 + (127.0 * (((x * x + y * y) as f32).sqrt() / 4.0 - t).sin())))
                            as i32
                            / 4;
                        *ptr = vv as u8;
                        ptr = ptr.offset(1);
                        *ptr = vv as u8;
                        ptr = ptr.offset(1);
                        *ptr = vv as u8;
                        ptr = ptr.offset(1);
                        *ptr = vv as u8;
                        ptr = ptr.offset(1);
                    }

                    dst = dst.offset(buffer.row_pitch() as isize);
                }
                end_modify_texture(handle, width, height, buffer);
            }
    }
}


extern "system" fn on_render_event(_: std::os::raw::c_int) {
    modify_texture_pixels();
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn GetRenderEventFunc() -> unity_native_plugin::graphics::RenderingEvent {
    Some(on_render_event)
}