#[macro_use]
extern crate glium;
extern crate gl;

use gl::types::*;
use glium::Surface;
use glium::glutin::{self, PossiblyCurrent};

use arrayfire as af;
use af_opencl_interop as afcl;
use ocl_interop;
use ocl::builders::ContextProperties;

use ocl::ProQue;

use takeable_option::Takeable;

use std::rc::Rc;
use std::cell::RefCell;
use std::os::raw::c_void;

fn main() {
    af::set_device(0);
    af::set_backend(af::Backend::OPENCL);
    af::info();

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let gl_window = cb
        .build_windowed(wb, &event_loop)
        .unwrap();
    let gl_window = unsafe { gl_window.make_current().unwrap() };

    unsafe {
        // Load GL Functions
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.0, 1.0, 0.333, 1.0);
    }

    let a = af::randu::<u8>(af::dim4!(8, 8, 3));
    let mut image = vec!(0u8; a.elements());
    a.host(&mut image);
    let img_ptr: *const _ = &image;

    // Choose platform & device(s) to use. Create a context, queue
    let platform = ocl::Platform::first().unwrap();
    println!("platform: {:?}", &platform);
    let af_did = afcl::get_device_id();
    let device_id = unsafe { ocl::core::DeviceId::from_raw(af_did) };
    let device = ocl::Device::by_idx_wrap(platform, af_did as usize).unwrap();
    println!("device id: {:?}", &device_id);
    println!("af device id: {:?}", &af_did);
    println!("device: {:?}", &device);

    let context = ocl_interop::get_context().expect("Cannot find GL's device in CL");
    println!("OpenGL context pointer: {:?}", &context);
    let queue = ocl::Queue::new(&context, device, None).unwrap();
    let dims = [256, 1, 1];
 
    println!("Pixel format of the window's GL context: {:?}", gl_window.get_pixel_format());


    let mut gl_tex: u32 = 0;
    let gl_tex_ptr: *mut u32 = &mut gl_tex;
    unsafe {
        gl::GenTextures(1, gl_tex_ptr);
        gl::BindTexture(gl::TEXTURE_2D, gl_tex);
    
        assert!(gl_tex != 0, "GL Texture cannot be empty");
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, 8, 8, 0, gl::RGB, gl::UNSIGNED_BYTE, img_ptr as *const std::ffi::c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }


    // let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image, (8, 8));
    // let opengl_texture = glium::texture::Texture2d::new(&display, raw_image).unwrap();
    // let opengl_texture = glium::texture::Texture2d::empty(&display, 4, 4).unwrap();

    let kernel_src = r#"
    __kernel void add(__global float* buffer, float scalar) {
        buffer[get_global_id(0)] += scalar;
    }
    "#;

    // let ocl_pq = ProQue::builder()
    //         .context(context)
    //         .build()
    //         .expect("Build ProQue");

    // Create a `Buffer`: TODO
    let cl_buffer = unsafe {
        ocl::core::create_from_gl_texture_2d(
            &context,
            gl::TEXTURE_2D,
            0,
            gl_tex,
            ocl::core::MEM_READ_WRITE
        ).unwrap()
    };
    // let cl_buffer = ocl::Buffer::<u8>::from_gl_buffer(&queue, None, gl_buff).unwrap();
    let mut vec = vec![0u8; dims[0]];
    // let cl_buffer = ocl::Buffer::builder().queue(queue.clone()).len(dims[0]).copy_host_slice(&vec).build().unwrap();
    // let cl_buffer = unsafe {
    //     ocl::core::create_buffer(
    //         &context,
    //         ocl::core::MEM_READ_WRITE | ocl::core::MEM_COPY_HOST_PTR,
    //         dims[0],
    //         Some(&vec),
    //     )
    //     .unwrap()
    // };

    // // get GL Objects
    // let mut acquire_globj_event: ocl::Event = ocl::Event::empty();
    // cl_buffer.cmd()
    //     .gl_acquire()
    //     .enew(&mut acquire_globj_event)
    //     .enq()
    //     .unwrap();
    // let mut vec_result = vec![0u8; dims[0]];

    // assert!(
    //     (dims[0] * std::mem::size_of::<u8>())
    //         == std::mem::size_of::<[u8; 8]>()
    // );

    // // Read results from the device into result_buffer's local vector:
    // // result_buffer.read(&mut vec_result).enq().unwrap();
    // let mut read_buffer_event: ocl::Event = ocl::Event::empty();
    // unsafe {
    //     cl_buffer
    //         .read(&mut vec_result)
    //         .block(false)
    //         .queue(&queue)
    //         .enew(&mut read_buffer_event)
    //         .enq()
    //         .unwrap();
    // }

    // // Release GL OBJs
    // cl_buffer.cmd()
    //     .gl_release()
    //     // .ewait(&kernel_run_event)
    //     .ewait(&read_buffer_event)
    //     .enq()
    //     .unwrap();

    queue.finish().unwrap(); //sync up before switching to arrayfire

    // Add custom device, context and associated queue to ArrayFire
    afcl::add_device_context(device_id.as_raw(), context.as_ptr(), queue.as_ptr());
    afcl::set_device_context(device_id.as_raw(), context.as_ptr());
    af::info();

    let mut af_buffer = af::Array::new_from_device_ptr(
        cl_buffer.as_ptr() as *mut u8,
        af::Dim4::new(&[dims[0] as u64, 1, 1, 1]),
    );
    println!("CL buffer ptr: {:?}", cl_buffer.as_ptr());
    println!("CL buffer ptr: {:?}", cl_buffer.as_ptr() as *mut u8);
    println!("Current active device: {}", af::get_device());
    af::af_print!("GPU Buffer before modification:", &af_buffer);

    af_buffer = af_buffer + 10u8;

    af::af_print!("GPU Buffer after modification:", &af_buffer);

    af::sync(af::get_device());
    let mut vec = vec![0u8; dims[0]];
    unsafe {
        let ptr = af_buffer.device_ptr();
        println!("ptr: {:?}", &ptr);
        let obuf = ocl::core::Mem::from_raw_copied_ptr(ptr);

        // Read results from the device into a vector:
       
    }
    println!("GPU buffer on host after ArrayFire operation: {:?}", vec);

    // let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image, (512, 512));
    // let opengl_texture = glium::texture::Texture2d::new(&display, raw_image).unwrap();

    // let vertex_buffer = {
    //     #[derive(Copy, Clone)]
    //     struct Vertex {
    //         position: [f32; 2],
    //         tex_coords: [f32; 2],
    //     }

    //     implement_vertex!(Vertex, position, tex_coords);

    //     glium::VertexBuffer::new(&display,
    //         &[
    //             Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] },
    //             Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 1.0] },
    //             Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
    //             Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] }
    //         ]
    //     ).unwrap()
    // };

    // // building the index buffer
    // let index_buffer = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TriangleStrip,
    //     &[1 as u16, 2, 0, 3]).unwrap();

    // // compiling shaders and linking them together
    // let program = glium::Program::from_source(&display, r"
    // #version 140
    // in vec2 position;
    // in vec2 tex_coords;
    // out vec2 v_tex_coords;
    // void main() {
    //     gl_Position = vec4(position, 0.0, 1.0);
    //     v_tex_coords = tex_coords;
    // }
    // ", r"
    // #version 140
    // uniform sampler2D tex;
    // in vec2 v_tex_coords;
    // out vec4 color;
    // void main() {
    //     color = texture(tex, v_tex_coords);
    // }
    // ", None).unwrap();

    event_loop.run(move |event, _, control_flow| {

        // Update texture
        

        // build uniforms
        // let uniforms = uniform! {
        //     tex: &opengl_texture
        // };

        // unsafe {
        //     gl::Clear(gl::COLOR_BUFFER_BIT);
        // }

        // drawing a frame
        // let mut target = display.draw();
        // // target.clear_color(0.0, 0.0, 0.0, 0.0);
        // target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
        // target.finish().unwrap();
        // let mut target = glium::Frame::new(gl_window, gl_window.window().inner_size().into());
        // target.clear_color(0.0, 1.0, 0.0, 1.0);
        // target.finish().unwrap();

        // gl_window.swap_buffers().unwrap();
        // std::thread::sleep(std::time::Duration::from_millis(5));

        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
                glutin::event::WindowEvent::Resized(size) => gl_window.resize(size),
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }
    });
}