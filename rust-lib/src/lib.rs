#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, glow};
use std::ffi::CStr;
use std::fmt::Debug;
use std::sync::OnceLock;
use std::{ffi::c_void, os::raw::c_char};

type LoaderFunc = extern "C" fn(name: *const c_char) -> *const c_void;

#[link(name = "vtktest")]
unsafe extern "C" {
    fn vtk_new(load: LoaderFunc);
    fn vtk_destroy();
    fn vtk_paint();
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([350.0, 380.0]),
        multisampling: 4,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "Eframe with VTK Widget",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
    .unwrap();

    0
}

struct MyApp {
    vtk_widget: VtkWidget,
    _angle: f32,
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc
            .gl
            .as_ref()
            .expect("You need to run eframe with the glow backend");

        let get_proc_address = cc
            .get_proc_address
            .expect("You need to run eframe with the glow backend");

        Self {
            vtk_widget: VtkWidget::new(gl, get_proc_address),
            _angle: 0.0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        use egui::load::SizedTexture;

        self.vtk_widget.paint(frame.gl().unwrap(), 0.0);

        // TODO: do this in a better place
        if self.vtk_widget.egui_texture_id.is_none() {
            self.vtk_widget.egui_texture_id =
                Some(frame.register_native_glow_texture(self.vtk_widget.texture));
        }
        let egui_texture_id = self.vtk_widget.egui_texture_id.unwrap();

        let vtk_img = egui::Image::from_texture(SizedTexture::new(
            egui_texture_id,
            [self.vtk_widget.width as f32, self.vtk_widget.height as f32],
        ));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("This thing is being painted using ");
                ui.hyperlink_to("vtk", "https://github.com/grovesNL/glow");
                ui.label(" (OpenGL).");
            });

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                ui.add(vtk_img);
            });
            ui.label("Drag to rotate!");
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.vtk_widget.destroy(gl);
        }
    }
}

struct VtkWidget {
    fbo: glow::NativeFramebuffer,
    texture: glow::NativeTexture,
    _depth_rb: glow::NativeRenderbuffer,
    width: i32,
    height: i32,
    egui_texture_id: Option<egui::TextureId>,
}

struct Holder {
    get_proc_address: &'static dyn Fn(&std::ffi::CStr) -> *const std::ffi::c_void,
}

impl Debug for Holder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Just some Holder...")
    }
}

unsafe impl Sync for Holder {}
unsafe impl Send for Holder {}

static CELL: OnceLock<Holder> = OnceLock::new();

pub extern "C" fn gl_load(name: *const c_char) -> *const c_void {
    let name = unsafe { CStr::from_ptr(name) };
    let holder = CELL.get().unwrap();
    (holder.get_proc_address)(name)
}

impl VtkWidget {
    fn new(
        gl: &glow::Context,
        get_proc_address: &dyn Fn(&std::ffi::CStr) -> *const std::ffi::c_void,
    ) -> Self {
        use glow::HasContext as _;

        CELL.set(Holder {
            get_proc_address: unsafe { std::mem::transmute(get_proc_address) },
        })
        .unwrap();
        unsafe { vtk_new(gl_load) };
        // FIXME: make the cell empty again

        let width = 300;
        let height = 300;

        unsafe {
            let fbo = gl.create_framebuffer().unwrap();
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(fbo));

            let texture = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );
            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(texture),
                0,
            );

            let depth_rb = gl.create_renderbuffer().unwrap();
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(depth_rb));
            gl.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH_COMPONENT24, width, height);
            gl.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                glow::RENDERBUFFER,
                Some(depth_rb),
            );

            assert_eq!(
                gl.check_framebuffer_status(glow::FRAMEBUFFER),
                glow::FRAMEBUFFER_COMPLETE
            );

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            VtkWidget {
                fbo,
                texture,
                _depth_rb: depth_rb,
                width,
                height,
                egui_texture_id: None,
            }
        }
    }

    fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;

        unsafe {
            gl.delete_framebuffer(self.fbo);
            gl.delete_texture(self.texture); // TODO: are we allowed?
            gl.delete_renderbuffer(self._depth_rb);
            vtk_destroy()
        }
    }

    fn paint(&self, gl: &glow::Context, _angle: f32) {
        use glow::HasContext as _;

        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
            vtk_paint();
            gl.bind_framebuffer(glow::FRAMEBUFFER, None);
        }
    }

    // TODO: add function to resize VTK
}
