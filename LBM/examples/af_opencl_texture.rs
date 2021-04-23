// extern crate gl;

// mod lbm;

// use af_opencl_interop as afcl;
// use arrayfire as af;

// use ocl::builders::ContextProperties;

// use ocl::ProQue;
// use gl::types::*;
// // use ocl_core::{ContextProperties, Event};

// use glutin::event::{Event, WindowEvent};
// use glutin::event_loop::{ControlFlow, EventLoop};
// use glutin::window::WindowBuilder;
// use glutin::ContextBuilder;
// use glutin::platform::windows::{RawContextExt, WindowExtWindows};
// use std::io::Write;
// use takeable_option::Takeable;

// // use lbmaf::computation_slow;

// // fn computation_slow() -> *mut std::ffi::c_void {
// //     set_device(0);
// //     set_backend(Backend::CUDA);

// //     let out = lbmD2Q9();
// //     let mut results = Box::new(vec!(0.0f32; out.elements()));
// //     out.host(&mut results);
    
// //     Box::into_raw(results) as *mut _
// // }

// // 3 triangles, 3 Vertices per triangle, 2 floats per vertex
// const BUFFER_LENGTH: usize = 18;
// const WINDOW_WIDTH: u32 = 800;
// const WINDOW_HEIGHT: u32 = 640;
// const MAX_FRAME_COUNT: u32 = 300; // 5 seconds

// const VERTEX_SRC: &'static str = include_str!("vertex.glsl");
// const FRAGMENT_SRC: &'static str = include_str!("fragment.glsl");

// fn main() {
//     af::set_device(0);
//     af::set_backend(af::Backend::OPENCL);
//     af::info();

//     // OpenGL/Windowing
//     let mut el = glutin::event_loop::EventLoop::new();
//     let wb = glutin::window::WindowBuilder::new();
//     let windowed_context = glutin::ContextBuilder::new()
//         .build_windowed(wb, &el)
//         .unwrap();

//     let windowed_context = unsafe { windowed_context.make_current().unwrap() };

//     // Choose platform & device(s) to use. Create a context, queue,
//     let platform_id = ocl::core::default_platform().unwrap();
//     println!("platform_id: {:?}", &platform_id);
//     let device_ids = ocl::core::get_device_ids(&platform_id, None, None).unwrap();
//     println!("device id: {:?}", &device_ids);
//     let device_id = device_ids[0];
//     // let context_properties = ContextProperties::new()
//     //     .platform(platform_id)
//     //     .interop_user_sync(true);
//     // let context =
//     //     ocl::core::create_context(Some(&context_properties), &[device_id], None, None).unwrap();
//     // Create an OpenCL context with the GL interop enabled
//     // let cgl_current_ctx = unsafe { CGLGetCurrentContext() };
//     // let cgl_share_grp = unsafe { CGLGetShareGroup(cgl_current_ctx) };
//     let ctx = ocl_interop::get_context().expect("Cannot find GL's device in CL");
//     println!("OpenGL ctx pointer: {:?}", &ctx);
//     let queue = ocl::core::create_command_queue(&ctx, &device_id, None).unwrap();
//     let dims = [8, 1, 1];

//     println!("Pixel format of the window's GL context: {:?}", windowed_context.get_pixel_format());

//     unsafe {
//         // Load GL Functions
//         gl::load_with(|symbol| windowed_context.get_proc_address(symbol) as *const _);
//         gl::ClearColor(0.0, 1.0, 0.333, 1.0);
//     }

//     // let data: [f32; 5] = [1.,2.,3.,4.,5.];
//     // let mut gl_buffer = glium::buffer::Buffer::new(&display, &data, glium::buffer::BufferType::TextureBuffer, glium::buffer::BufferMode::Default);
//     println!("hi");
//     let mut gl_buff: GLuint = 0;
//     unsafe {
//         gl::GenBuffers(1, &mut gl_buff);
//     }

//     let kernel_src = r#"
//     __kernel void multiply_by_scalar(
//         __private float coeff,
//         __global float * src,
//         __global float*  res)
//     {
//         uint const idx = get_global_id(0);
//         res[idx] = src[idx] * coeff;
//         }

//         __kernel void fill_vbo(__global float* vbo){
//         int id=get_global_id(0);
//         vbo[id]=(id%6)/3+(id%2)*(id/6);
//         vbo[id]/=3;
//     }
//     "#;

//     let ocl_pq = ProQue::builder()
//             .context(ctx)
//             .src(kernel_src)
//             .build()
//             .expect("Build ProQue");

//     // Create a `Buffer`:
//     // let mut vec = vec![0.0f32; dims[0]];
//     let cl_buffer = ocl::Buffer::<f32>::from_gl_buffer(ocl_pq.queue(), None, gl_buff).unwrap();
//         // ocl::core::create_buffer(
//         //     &context,
//         //     ocl::core::MEM_READ_WRITE | ocl::core::MEM_COPY_HOST_PTR,
//         //     dims[0],
//         //     Some(&vec),
//         // )
//         // .unwrap()
//     ocl::core::finish(&queue).unwrap(); //sync up before switching to arrayfire

//     // // Add custom device, context and associated queue to ArrayFire
//     // afcl::add_device_context(device_id.as_raw(), context.as_ptr(), queue.as_ptr());
//     // afcl::set_device_context(device_id.as_raw(), context.as_ptr());
//     // af::info();

//     // let mut af_buffer = af::Array::new_from_device_ptr(
//     //     buffer.as_ptr() as *mut f32,
//     //     af::Dim4::new(&[dims[0] as u64, 1, 1, 1]),
//     // );

//     // af::af_print!("GPU Buffer before modification:", af_buffer);

//     // // af_buffer = af_buffer + 10f32;

//     // // ocl::prm::cl_GLuint

//     // // af::sync(af::get_device());
//     // // unsafe {
//     // //     let ptr = af_buffer.device_ptr();
//     // //     println!("ptr: {:?}", &ptr);
//     // //     let obuf = ocl::core::Mem::from_raw_copied_ptr(ptr);

//     // //     // Read results from the device into a vector:
//     // //     ocl::core::enqueue_read_buffer(
//     // //         &queue,
//     // //         &obuf,
//     // //         true,
//     // //         0,
//     // //         &mut vec,
//     // //         None::<Event>,
//     // //         None::<&mut Event>,
//     // //     )
//     // //     .unwrap();
//     // // }
//     // println!("GPU buffer on host after ArrayFire operation: {:?}", vec);

//     // // Remove device from ArrayFire management towards Application Exit
//     // af::set_device(0); // Cannot pop when in Use, hence switch to another device
//     // afcl::delete_device_context(device_id.as_raw(), context.as_ptr());
//     // println!("GPU buffer on host after ArrayFire operation: {:?}", vec);

//     println!("hi");

//     el.run(move |event, _, control_flow| {
//         println!("el {:?}", event);
//         *control_flow = ControlFlow::Wait;

//         match event {
//             Event::LoopDestroyed => {
//                 Takeable::take(&mut windowed_context); // Make sure it drops first
//                 return;
//             }
//             Event::WindowEvent { event, .. } => match event {
//                 WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
//                 WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
//                 _ => (),
//             },
//             Event::RedrawRequested(_) => {
//                 gl.draw_frame( [1.0, 0.5, 0.7, 1.0]);
//                 windowed_context.swap_buffers().unwrap();
//             }
//             _ => (),
//         }
//     });
// }

fn main() {
    
}