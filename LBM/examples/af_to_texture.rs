#[macro_use]
extern crate glium;
use glium::{glutin, Surface};
use arrayfire as af;

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let a = af::randu::<u8>(af::dim4!(512, 512, 4));
    let mut image = vec!(0u8; a.elements());
    a.host(&mut image);

    let raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image, (512, 512));
    let opengl_texture = glium::texture::Texture2d::new(&display, raw_image).unwrap();

    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            tex_coords: [f32; 2],
        }

        implement_vertex!(Vertex, position, tex_coords);

        glium::VertexBuffer::new(&display,
            &[
                Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] },
                Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 1.0] },
                Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
                Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] }
            ]
        ).unwrap()
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display, glium::index::PrimitiveType::TriangleStrip,
        &[1 as u16, 2, 0, 3]).unwrap();

    // compiling shaders and linking them together
    let program = glium::Program::from_source(&display, r"
    #version 140
    in vec2 position;
    in vec2 tex_coords;
    out vec2 v_tex_coords;
    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
        v_tex_coords = tex_coords;
    }
    ", r"
    #version 140
    uniform sampler2D tex;
    in vec2 v_tex_coords;
    out vec4 color;
    void main() {
        color = texture(tex, v_tex_coords);
    }
    ", None).unwrap();

    event_loop.run(move |event, _, control_flow| {
        // Update texture
        let b = af::randu::<u8>(af::dim4!(512, 512, 4));
        b.host(&mut image);
        let new_raw_image = glium::texture::RawImage2d::from_raw_rgba_reversed(&image, (512, 512));
        opengl_texture.write(glium::Rect { bottom: 0, left: 0, width: 512, height: 512 }, new_raw_image);

        // build uniforms
        let uniforms = uniform! {
            tex: &opengl_texture
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();

        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                },
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