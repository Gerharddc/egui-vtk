#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use glow::*;
use glutin::context::GlProfile;
use glutin::display::Display;
use glutin::prelude::GlDisplay;
use std::sync::OnceLock;
use std::{ffi::CStr, ffi::c_void, os::raw::c_char};

static DISPLAY_CELL: OnceLock<Display> = OnceLock::new();

type LoaderFunc = extern "C" fn(name: *const c_char) -> *const c_void;

#[link(name = "vtktest")]
unsafe extern "C" {
    fn vtk_new(load: LoaderFunc);
    fn vtk_destroy();
    fn vtk_paint();
}

pub extern "C" fn gl_load(name: *const c_char) -> *const c_void {
    let name = unsafe { CStr::from_ptr(name) };
    let gl_display = DISPLAY_CELL.get().unwrap();
    let ptr = gl_display.get_proc_address(&name);
    ptr
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    unsafe {
        let (gl, gl_surface, gl_context, shader_version, _window, event_loop) = {
            use glutin::{
                config::{ConfigTemplateBuilder, GlConfig},
                context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext},
                display::{GetGlDisplay, GlDisplay},
                surface::{GlSurface, SwapInterval},
            };
            use glutin_winit::{DisplayBuilder, GlWindow};
            use raw_window_handle::HasRawWindowHandle;
            use std::num::NonZeroU32;

            let event_loop = winit::event_loop::EventLoopBuilder::new().build().unwrap();
            let window_builder = winit::window::WindowBuilder::new()
                .with_title("Hello triangle!")
                .with_inner_size(winit::dpi::LogicalSize::new(300.0, 300.0));

            let template = ConfigTemplateBuilder::new();

            let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

            let (window, gl_config) = display_builder
                .build(&event_loop, template, |configs| {
                    configs
                        .reduce(|accum, config| {
                            if config.num_samples() > accum.num_samples() {
                                config
                            } else {
                                accum
                            }
                        })
                        .unwrap()
                })
                .unwrap();

            let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());

            let gl_display = gl_config.display();
            let context_attributes = ContextAttributesBuilder::new()
                .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version {
                    major: 4,
                    minor: 1,
                })))
                .with_profile(GlProfile::Compatibility)
                .with_debug(true)
                .build(raw_window_handle);

            let not_current_gl_context = gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap();

            let window = window.unwrap();

            let attrs = window.build_surface_attributes(Default::default());
            let gl_surface = gl_display
                .create_window_surface(&gl_config, &attrs)
                .unwrap();

            let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

            let gl = glow::Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s));

            DISPLAY_CELL.set(gl_display).unwrap();
            vtk_new(gl_load);

            gl_surface
                .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
                .unwrap();

            (
                gl,
                gl_surface,
                gl_context,
                "#version 410",
                window,
                event_loop,
            )
        };

        let vertex_array = gl
            .create_vertex_array()
            .expect("Cannot create vertex array");
        gl.bind_vertex_array(Some(vertex_array));

        let program = gl.create_program().expect("Cannot create program");

        let (vertex_shader_source, fragment_shader_source) = (
            r#"const vec2 verts[3] = vec2[3](
                vec2(0.5f, 1.0f),
                vec2(0.0f, 0.0f),
                vec2(1.0f, 0.0f)
            );
            out vec2 vert;
            void main() {
                vert = verts[gl_VertexID];
                gl_Position = vec4(vert - 0.5, 0.0, 1.0);
            }"#,
            r#"precision mediump float;
            in vec2 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 0.5, 1.0);
            }"#,
        );

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        let mut shaders = Vec::with_capacity(shader_sources.len());

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        gl.use_program(Some(program));
        gl.clear_color(0.1, 0.2, 0.3, 1.0);

        use glutin::prelude::GlSurface;
        use winit::event::{Event, WindowEvent};
        let _ = event_loop.run(move |event, elwt| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => {
                        vtk_destroy();
                        elwt.exit();
                    }
                    WindowEvent::RedrawRequested => {
                        gl.clear(glow::COLOR_BUFFER_BIT);
                        gl.draw_arrays(glow::TRIANGLES, 0, 3);
                        vtk_paint();
                        gl_surface.swap_buffers(&gl_context).unwrap();
                    }
                    _ => (),
                }
            }
        });
    }

    0
}
