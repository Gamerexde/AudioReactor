#[macro_use]
extern crate glium;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use glium::{glutin, Surface};

use soloud::*;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

fn triangle_vertexs() -> Vec<Vertex> {
    let mut wave_vertex: Vec<Vertex> = Vec::with_capacity(258);

    wave_vertex.push(Vertex { position: [-1.0, -1.0] });
    wave_vertex.push(Vertex { position: [1.0, -1.0] });

    for i in 0..256 {
        wave_vertex.push(Vertex { position: [i as f32 / 256_f32, -1.0] });
    }

    return wave_vertex;
}

fn main() {
    implement_vertex!(Vertex, position);

    let mut event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let shape = triangle_vertexs();
    let shape_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);

    shape_buffer.write(&shape);

    let wave: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::with_capacity(256)));

    // Initialize values in the empty vec otherwise stuff can go bad really fast...
    for i in 0..256 {
        let wave = wave.clone();

        let mut wave = wave.lock().unwrap();

        wave.push(0_f32);
    }

    thread::spawn({
        let wave = wave.clone();

        move || {
            let mut sl = Soloud::default().unwrap();

            sl.set_visualize_enable(true);

            let wav = Wav::from_path(&std::path::Path::new("no path for now uwu")).unwrap();

            sl.play(&wav);

            while sl.voice_count() > 0 {
                thread::sleep(Duration::from_millis(5));

                let mut wave = wave.lock().unwrap();
                *wave = sl.wave();
            }
        }
    });

    let vertex_shader_src = r#"
    #version 400
    in vec2 position;

    out float brightness;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
    }
"#;

    let fragment_shader_src = r#"
    #version 400
    out vec4 color;

    void main() {
        color = vec4(1.0, 1.0, 1.0, 1.0);
    }
"#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    event_loop.run({

        let wave = wave.clone();

        move |ev, _, control_flow| {
            match ev {
                glutin::event::Event::WindowEvent { event, .. } => match event {
                    glutin::event::WindowEvent::CloseRequested => {
                        *control_flow = glutin::event_loop::ControlFlow::Exit;
                        return;
                    },
                    _ => return,
                },
                _ => (),
            }

            let next_frame_time = std::time::Instant::now() + Duration::from_nanos(16_666_667);
            *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);

            let wave = wave.lock().unwrap();


            let mut wave_vertex: Vec<Vertex> = Vec::with_capacity(258);

            wave_vertex.push(Vertex { position: [-1.0, -1.0] });
            wave_vertex.push(Vertex { position: [1.0, -1.0] });

            for (i, val) in wave.iter().enumerate() {
                wave_vertex.push(Vertex { position: [-1_f32 + (i as f32 / 129_f32), val.clone()] });
            }

            shape_buffer.write(&wave_vertex);

            /*
            let items = 1;

            let mut average: f32 = 0_f32;

            for i in 0..items {
                average += wave[i] + f32::abs(wave[i] * 2_f32);
            }

            average = average / 1_f32;

             */

            target.draw(&shape_buffer, &indices, &program, &glium::uniforms::EmptyUniforms, &Default::default()).unwrap();
            target.finish().unwrap();
        }
    });
}
