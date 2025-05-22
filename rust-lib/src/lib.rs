#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, egui_glow, glow};
use egui::mutex::Mutex;
use std::sync::Arc;

#[link(name = "vtktest")]
unsafe extern "C" {
    fn vtk_new();
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
        stencil_buffer: 8,
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
        Self {
            vtk_widget: Arc::new(Mutex::new(VtkWidget::new(gl))),
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
                rotating_triangle.lock().paint(painter.gl(), angle);
            })),
        };
        ui.painter().add(callback);
    }
}

struct VtkWidget {}

impl VtkWidget {
    fn new(_gl: &glow::Context) -> Self {
        //use glow::HasContext as _;
        unsafe {
            vtk_new();
        };
        VtkWidget {}
    }

    fn destroy(&self, _gl: &glow::Context) {
        //use glow::HasContext as _;
        unsafe { vtk_destroy() }
    }

    fn paint(&self, _gl: &glow::Context, _angle: f32) {
        //use glow::HasContext as _;
        unsafe { vtk_paint() }
    }

    // TODO: add function to resize VTK
}
