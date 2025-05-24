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
        let (gl, gl_surface, gl_context, _window, event_loop) = {
            use glutin::{
                config::{ConfigTemplateBuilder, GlConfig},
                context::{ContextAttributesBuilder, NotCurrentGlContext},
                display::{GetGlDisplay, GlDisplay},
                surface::{GlSurface, SwapInterval},
            };
            use glutin_winit::{DisplayBuilder, GlWindow};
            use raw_window_handle::HasRawWindowHandle;
            use std::num::NonZeroU32;

            let event_loop = winit::event_loop::EventLoopBuilder::new().build().unwrap();
            let window_builder = winit::window::WindowBuilder::new()
                .with_title("Hello VTK!")
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

            (gl, gl_surface, gl_context, window, event_loop)
        };

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
                        gl.enable(DEPTH_TEST);

                        gl.clear_color(0.0, 0.0, 0.0, 1.0);
                        gl.clear_depth(1.0);
                        gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);

                        vtk_paint();

                        gl_surface.swap_buffers(&gl_context).unwrap();
                    }
                    WindowEvent::Resized(physical_size) => {
                        gl.viewport(
                            0,
                            0,
                            physical_size.width as i32,
                            physical_size.height as i32,
                        );
                    }
                    _ => (),
                }
            }
        });
    }

    0
}
