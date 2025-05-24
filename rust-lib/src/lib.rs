#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui::Rect;
use eframe::{egui, egui_glow, glow};
use egui::mutex::Mutex;
use std::ffi::CStr;
use std::fmt::Debug;
use std::sync::Arc;
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
        depth_buffer: 24,
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
    /// Behind an `Arc<Mutex<…>>` so we can pass it to [`egui::PaintCallback`] and paint later.
    vtk_widget: Arc<Mutex<VtkWidget>>,
    angle: f32,
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
            vtk_widget: Arc::new(Mutex::new(VtkWidget::new(gl, get_proc_address))),
            angle: 0.0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("The triangle is being painted using ");
                ui.hyperlink_to("glow", "https://github.com/grovesNL/glow");
                ui.label(" (OpenGL).");
            });

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
            ui.label("Drag to rotate!");
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.vtk_widget.lock().destroy(gl);
        }
    }
}

impl MyApp {
    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

        self.angle += response.drag_motion().x * 0.01;

        // Clone locals so we can move them into the paint callback:
        let angle = self.angle;
        let rotating_triangle = self.vtk_widget.clone();

        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                rotating_triangle.lock().paint(painter.gl(), angle, rect);
            })),
        };
        ui.painter().add(callback);
    }
}

struct VtkWidget {}

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
        _gl: &glow::Context,
        get_proc_address: &dyn Fn(&std::ffi::CStr) -> *const std::ffi::c_void,
    ) -> Self {
        //use glow::HasContext as _;

        CELL.set(Holder {
            get_proc_address: unsafe { std::mem::transmute(get_proc_address) },
        })
        .unwrap();
        unsafe { vtk_new(gl_load) };

        // FIXME: make the cell empty again

        VtkWidget {}
    }

    fn destroy(&self, _gl: &glow::Context) {
        //use glow::HasContext as _;
        unsafe { vtk_destroy() }
    }

    fn paint(&self, gl: &glow::Context, _angle: f32, rect: Rect) {
        use glow::HasContext as _;

        println!("Rect: {:#?}", rect);
        println!(
            "L: {}, B: {}, W: {}, H: {}",
            rect.left(),
            rect.bottom(),
            rect.width(),
            rect.height()
        );

        unsafe {
            // Query current viewport settings
            let mut viewport = [0; 4];
            gl.get_parameter_i32_slice(glow::VIEWPORT, &mut viewport);

            // viewport is a slice with 4 values: [x, y, width, height]
            let x = viewport[0];
            let y = viewport[1];
            let width = viewport[2];
            let height = viewport[3];

            println!(
                "Current viewport: x={}, y={}, width={}, height={}",
                x, y, width, height
            );

            gl.enable(glow::DEPTH_TEST);

            gl.enable(glow::SCISSOR_TEST);
            gl.scissor(0, 0, 100, 100);
            gl.clear_color(0.0, 1.0, 0.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.disable(glow::SCISSOR_TEST);

            gl.clear_depth(1.0);
            gl.clear(glow::DEPTH_BUFFER_BIT);

            vtk_paint();

            gl.disable(glow::DEPTH_TEST);

            // Query current viewport settings
            let mut viewport = [0; 4];
            gl.get_parameter_i32_slice(glow::VIEWPORT, &mut viewport);

            // viewport is a slice with 4 values: [x, y, width, height]
            let x = viewport[0];
            let y = viewport[1];
            let width = viewport[2];
            let height = viewport[3];

            println!(
                "Current viewport: x={}, y={}, width={}, height={}",
                x, y, width, height
            );
        };
    }

    // TODO: add function to resize VTK
}
