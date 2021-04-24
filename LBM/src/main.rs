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

extern crate image;
use image::GenericImage;

use takeable_option::Takeable;

use std::rc::Rc;
use std::cell::RefCell;
use std::io::Cursor;
use std::os::raw::c_void;

fn main() {
    af::set_device(0);
    af::set_backend(af::Backend::OPENCL);
    af::info();

    let event_loop = glutin::event_loop::EventLoop::new();
    let size: glutin::dpi::LogicalSize<u32> = (800, 800).into();
    let wb = glutin::window::WindowBuilder::new()
        .with_inner_size(size)
        .with_title("Hellow world");
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

    println!("Pixel format of the window's GL context: {:?}", gl_window.get_pixel_format());

    // Arrayfire stuff
    let af_buffer = af::constant(0u8, af::dim4!(8, 8, 4));

    let af_did = afcl::get_device_id();
    let af_ctx = afcl::get_context(false);
    let af_que = afcl::get_queue(false);

    let _devid = unsafe { ocl::core::DeviceId::from_raw(af_did) };
    let context = unsafe { ocl::core::Context::from_raw_copied_ptr(af_ctx) };
    let queue = unsafe { ocl::core::CommandQueue::from_raw_copied_ptr(af_que) };

    let texture_data = vec![0u8; 256];
    let texture = unsafe {
        let mut texture = 0;
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture); 
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
        assert!(texture != 0, "GL Texture cannot be empty");
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, 8, 8, 0, gl::RGBA, gl::UNSIGNED_BYTE, texture_data.as_ptr() as *const u8 as *const c_void);
        gl::GenerateMipmap(gl::TEXTURE_2D);
        texture
    };

    // Create a `Buffer`:
    let cl_buffer = unsafe {
        ocl::core::create_from_gl_texture_2d(
            &context,
            gl::TEXTURE_2D,
            0,
            texture,
            ocl::core::MEM_READ_WRITE,
        ).unwrap()
    };

    ocl::core::finish(&queue).unwrap(); //sync up before switching to arrayfire
    af::info();

    let mut texture_buffer = af::Array::new_from_device_ptr(
        cl_buffer.as_ptr() as *mut u8,
        af::dim4!(8, 8, 4),
    );
    println!("CL buffer ptr: {:?}", cl_buffer.as_ptr());
    println!("Current active device: {}", af::get_device());

    af::af_print!("GPU Buffer before modification:", &af_buffer);

    texture_buffer = af_buffer + 10u8;

    af::af_print!("GPU Buffer after modification:", &texture_buffer);

    af::sync(af::get_device());
    let mut vec = vec![0u8; 256];
    unsafe {
        let ptr = texture_buffer.device_ptr();
        let obuf = ocl::core::Mem::from_raw_copied_ptr(ptr);

        ocl::core::enqueue_read_buffer(
            &queue,
            &obuf,
            true,
            0,
            &mut vec,
            None::<ocl::core::Event>,
            None::<&mut ocl::core::Event>
        ).unwrap();
       
    }
    println!("GPU buffer on host after ArrayFire operation: {:?}", vec);

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
        unsafe { gl::Viewport(0, 0, 800, 800) }
        // Update texture
        // TODO

        // build uniforms
        // let uniforms = uniform! {
        //     tex: &texture
        // };

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // drawing a frame
        // TODO

        gl_window.swap_buffers().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));

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