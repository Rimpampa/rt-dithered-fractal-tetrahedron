use glutin::dpi::LogicalSize;
use glutin::event::{DeviceEvent, ElementState, Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;

mod graphics;

use graphics::DepthBuffer;
use graphics::FragmentShader;
use graphics::Framebuffer;
use graphics::Program;
use graphics::Texture;
use graphics::VertexArrayObject;
use graphics::VertexBufferObject;
use graphics::VertexShader;

#[allow(unused)]
mod fractal;
use fractal::*;

#[macro_use]
mod dithering;
use dithering::dither;
use dithering::Color;
use dithering::Palette;
use dithering::WorkerPool;

use std::collections::VecDeque;
use std::convert::TryFrom;
use std::mem::size_of;
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;

mod math {
    pub use std::f32::consts::*;
    pub const TWO_THIRDS_PI: f32 = FRAC_PI_3 * 2.0;
    pub const FOUR_THIRDS_PI: f32 = TWO_THIRDS_PI * 2.0;
    pub const TWICE_PI: f32 = PI * 2.0;
}

fn main() -> Result<(), String> {
    // Create the event loop
    let el = EventLoop::new();
    // Create the window builder
    let wb = WindowBuilder::new()
        .with_title("Glutin Triangle") // Set the title of the window
        .with_inner_size(LogicalSize::new(500.0, 500.0)) // Set the size of the window
        .with_transparent(true); // Set the window to be trasparent
                                 // Create the window context from the winow builder and the event loop
    let wc = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    // Set the window context as the current context
    let window = unsafe { wc.make_current().unwrap() };
    // Load the opengl functions
    gl::load_with(|symbol| window.context().get_proc_address(symbol) as *const _);
    unsafe {
        // Enable depth testing
        gl::DepthFunc(gl::LESS);
        gl::Enable(gl::DEPTH_TEST);
    }
    // FRACTAL PROGRAM

    // Load the vertex shader form the file and compile it
    let vs = unsafe { VertexShader::from_file(Path::new("shaders/fractal.vert"))? };
    // Load the fragment shader form the file and compile it
    let fs = unsafe { FragmentShader::from_file(Path::new("shaders/fractal.frag"))? };
    // Link the shaders to the program and compile it
    let fractal_program = unsafe { Program::new(&vs, None, &fs)? };

    drop(vs);
    drop(fs);

    // Load the vertex shader form the file and compile it
    let vs = unsafe { VertexShader::from_file(Path::new("shaders/texture.vert"))? };
    // Load the fragment shader form the file and compile it
    let fs = unsafe { FragmentShader::from_file(Path::new("shaders/texture.frag"))? };
    // Link the shaders to the program and compile it
    let texture_program = unsafe { Program::new(&vs, None, &fs)? };

    drop(vs);
    drop(fs);

    unsafe { Program::bind(&fractal_program) };

    // Number of iterations
    const ITERATIONS: u32 = 4;
    let iterations = std::env::args()
        .nth(1)
        .as_deref()
        .map(u32::from_str)
        .map(|v| v.unwrap_or(ITERATIONS))
        .unwrap_or(ITERATIONS);

    // The number of tetrahedrons generated is equal to four to the nth power, where n is the number of iterations
    let size = 4usize.pow(iterations);
    // Create a double-ended queue for storing the tetrahedrons
    let mut vec = VecDeque::with_capacity(size);
    // Insert the first tetrahedron in the queue
    vec.push_back(Tetrahedron::regular(Point::new(0.0, -0.7, 0.0), 1.4, 0.0));
    // For each iteration:
    for i in 0..iterations {
        // For each tetrahedron:
        for _ in 0..4usize.pow(i) {
            // Remove it from the list
            let tetra = vec.pop_front().unwrap();
            // Split it into four parts
            let [a, b, c, d] = tetra.sierpinski_split();
            // Push them at the end of the queue
            vec.push_back(a);
            vec.push_back(b);
            vec.push_back(c);
            vec.push_back(d);
        }
    }
    // Trasform the deque into a vector
    let vec: Vec<Tetrahedron> = vec.into_iter().collect();

    let fra_vao = unsafe { VertexArrayObject::new() };
    let tex_vao = unsafe { VertexArrayObject::new() };
    unsafe { VertexArrayObject::bind(&fra_vao) };

    let vbo = unsafe { VertexBufferObject::new(size, Some(&vec)) };
    unsafe { VertexBufferObject::bind(&vbo) };

    unsafe {
        let coord_loc = fractal_program.vertex_attrib_location("coord")? as u32;
        VertexArrayObject::f32_attrib_format(coord_loc, 3, size_of::<Point>(), 0);
        VertexBufferObject::unbind();
        VertexArrayObject::unbind();
    }

    let mut texture = unsafe { Texture::new(250, 250, &[0; 250 * 250 * 4]) };
    unsafe { Texture::bind(&texture) };

    let mut depthbuffer = unsafe { DepthBuffer::new(texture.width(), texture.height()) };
    let framebuffer = unsafe { Framebuffer::new(&texture, Some(&depthbuffer))? };

    // Get the locations of the rotation matrix uniform
    let mat_loc = unsafe { fractal_program.uniform_location("mat")? };
    // Rotation matrix initialized at 0 degrees
    let mut _matrix = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let mut angle = 0f32;

    // Get the location of the fractal color uniform
    let color_loc = unsafe { fractal_program.uniform_location("color")? };
    // Initialize the color to black
    let mut _color = Point::new(0.0, 0.0, 0.0);
    unsafe {
        // Initialize the data of the two uniforms
        gl::Uniform3f(color_loc, _color.x, _color.y, _color.z);
        gl::UniformMatrix3fv(mat_loc, 1, gl::FALSE, _matrix.as_ptr());
    }

    let mut time = 0.0;
    let mut counter = 0;

    /*
        NOTE:
        As the threads need to sync with each other and with the pool, using a number of threads
        that exceeds the number of cpus in your system will make the os schedule the threads thus
        blocking the system as that thread must continue working in order for the others to finish.
        The thread pool, also, has to run on some thread thus reducing the number of simultaneous
        workers to the number of the cpus minus one.
    */
    let mut pool = WorkerPool::new(num_cpus::get() - 1);
    let palette = Palette::new([
        rgb![#ffffff],
        rgb![#ff0000],
        rgb![#00ff00],
        rgb![#0000ff],
        rgb![#000000],
    ]);

    let mut win_width = 500;
    let mut win_height = 500;

    let mut pressing = false;

    // Initialize the update flag which tells whether or not the screen should be updated
    let mut update = true;
    let mut elapsed = Instant::now();
    // Run the event loop
    el.run(move |event, _, control_flow| {
        let start = Instant::now();

        *control_flow = ControlFlow::Poll;
        match event {
            #[allow(clippy::single_match)]
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::Button { button: 1, state } => {
                    pressing = state == ElementState::Pressed;
                    if pressing {
                        unsafe {
                            texture.resize(win_width, win_height);
                            depthbuffer.resize(texture.width(), texture.height());
                        }
                    } else {
                        unsafe {
                            texture.resize(win_width / 2, win_height / 2);
                            depthbuffer.resize(texture.width(), texture.height());
                        }
                    }
                }
                _ => {}
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    window.resize(size);
                    win_width = size.width as u32;
                    win_height = size.height as u32;
                    unsafe {
                        texture.resize(win_width / 2, win_height / 2);
                        depthbuffer.resize(texture.width(), texture.height());
                    }
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },
            _ => {}
        }
        // If at least twenty milliseconds have passed rotate the fractal and change its color
        if elapsed.elapsed().as_secs_f32() > 0.02 {
            // Increment the angle
            angle += 0.01;
            if angle > math::TWICE_PI {
                angle -= math::TWICE_PI;
            }
            let cos = angle.cos();
            let sin = angle.sin();
            // Calculate the new rotation matrix
            _matrix = [cos, 0.0, -sin, 0.0, 1.0, 0.0, sin, 0.0, cos];
            unsafe {
                // Update the uniform which stores the matrix
                gl::UniformMatrix3fv(mat_loc as i32, 1, gl::FALSE, _matrix.as_ptr());
            }
            // Reset the timer
            elapsed = Instant::now();
            let angle = (angle * 10.0).rem_euclid(math::TWICE_PI);
            // Calculate the new color based on the angle
            if angle < math::TWO_THIRDS_PI {
                let angle = angle / math::TWO_THIRDS_PI;
                let angle = angle * math::FRAC_PI_2;
                _color = Point::new(angle.cos(), angle.sin(), 0.0);
            } else if angle < math::FOUR_THIRDS_PI {
                let angle = angle - math::TWO_THIRDS_PI;
                let angle = angle / math::TWO_THIRDS_PI;
                let angle = angle * math::FRAC_PI_2;
                _color = Point::new(0.0, angle.cos(), angle.sin());
            } else {
                let angle = angle - math::FOUR_THIRDS_PI;
                let angle = angle / math::TWO_THIRDS_PI;
                let angle = angle * math::FRAC_PI_2;
                _color = Point::new(angle.sin(), 0.0, angle.cos());
            }
            unsafe {
                // Update the color uniform
                gl::Uniform3f(color_loc as i32, _color.x, _color.y, _color.z);
            }
            update = true; // Notify the change
        }
        // If the screen needs to be updated
        if update {
            update = false; // Reset the flag
            unsafe {
                Framebuffer::bind(&framebuffer);
                gl::Enable(gl::DEPTH_TEST);
                VertexArrayObject::bind(&fra_vao);
                // Clear the previus image
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                gl::Viewport(0, 0, texture.width() as i32, texture.height() as i32);
                // Draw the new one
                gl::DrawArrays(gl::TRIANGLES, 0, 12 * size as i32);

                if !pressing {
                    // let start = Instant::now();

                    let mut pixels = texture.pixels();
                    let mut colors: Vec<Color> = pixels
                        .chunks_exact_mut(4)
                        .map(|v| &mut v[..3])
                        .map(<&mut [u8; 3]>::try_from)
                        .map(Result::unwrap)
                        .map(Color::from)
                        .collect();
                    dither(
                        &mut colors,
                        texture.width() as usize,
                        texture.height() as usize,
                        &palette,
                        pool.scope(),
                    );
                    texture.update(&pixels);

                    // time += start.elapsed().as_secs_f64();
                    // counter += 1;
                    // if counter == 50 {
                    //     println!("dither: {}", time / 50.0);
                    //     counter = 0;
                    //     time = 0.0;
                    // }
                }

                Framebuffer::unbind();
                gl::Disable(gl::DEPTH_TEST);
                Program::bind(&texture_program);
                VertexArrayObject::bind(&tex_vao);
                Texture::bind(&texture);
                gl::Viewport(0, 0, win_width as i32, win_height as i32);
                gl::DrawArrays(gl::TRIANGLES, 0, 6);

                Program::bind(&fractal_program);
                opengl_error();
            }
            // Swap the window buffers
            window.swap_buffers().unwrap();
            time += start.elapsed().as_secs_f64();
            counter += 1;
            if counter == 50 {
                time /= 50.0;
                println!("fps: {}", 1.0 / time);
                counter = 1;
            }
        }
    });
}

fn opengl_error() {
    let error = unsafe { gl::GetError() };
    debug_assert!(error == 0, "{}", error);
}
