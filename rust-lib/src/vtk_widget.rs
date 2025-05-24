use eframe::{egui, glow};
use std::ffi::CStr;
use std::fmt::Debug;
use std::sync::OnceLock;
use std::{ffi::c_void, os::raw::c_char};

type LoaderFunc = extern "C" fn(name: *const c_char) -> *const c_void;

#[link(name = "vtktest")]
unsafe extern "C" {
    fn vtk_new(load: LoaderFunc, width: i32, height: i32);
    fn vtk_destroy();
    fn vtk_paint();
    fn vtk_is_dirty() -> bool;

    fn vtk_mouse_move(x: i32, y: i32);
    fn vtk_update_mouse_down(primary: bool, secondary: bool, middle: bool);
    fn vtk_mouse_wheel(delta: i32);
    fn vtk_set_size(width: i32, height: i32);
}

pub struct VtkWidget {
    fbo: glow::NativeFramebuffer,
    texture: glow::NativeTexture,
    _depth_rb: glow::NativeRenderbuffer,
    width: i32,
    height: i32,
    egui_texture_id: Option<egui::TextureId>,
}

struct ProcFnHolder {
    get_proc_address: &'static dyn Fn(&std::ffi::CStr) -> *const std::ffi::c_void,
}

impl Debug for ProcFnHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Nothing to see here...")
    }
}

unsafe impl Sync for ProcFnHolder {}
unsafe impl Send for ProcFnHolder {}

static PROC_FN_CELL: OnceLock<ProcFnHolder> = OnceLock::new();

pub extern "C" fn gl_load(name: *const c_char) -> *const c_void {
    let name = unsafe { CStr::from_ptr(name) };
    let holder = PROC_FN_CELL.get().unwrap();
    (holder.get_proc_address)(name)
}

impl VtkWidget {
    pub fn new(
        gl: &glow::Context,
        get_proc_address: &dyn Fn(&std::ffi::CStr) -> *const std::ffi::c_void,
    ) -> Self {
        use glow::HasContext as _;

        let width = 300;
        let height = 300;

        PROC_FN_CELL
            .set(ProcFnHolder {
                get_proc_address: unsafe { std::mem::transmute(get_proc_address) },
            })
            .unwrap();

        unsafe {
            vtk_new(gl_load, width, height);

            // TODO: review this FBO code

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

            // FIXME: clear PROC_FN_CELL since it is not safe to use after this point...

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

    // It is only safe to call this when eframe exists since it likely owns the texture
    pub unsafe fn destroy(&self, gl: &glow::Context) {
        use glow::HasContext as _;

        unsafe {
            gl.delete_framebuffer(self.fbo);
            gl.delete_texture(self.texture);
            gl.delete_renderbuffer(self._depth_rb);
            vtk_destroy()
        }
    }

    pub fn paint_if_dirty(&self, gl: &glow::Context) {
        use glow::HasContext as _;

        unsafe {
            if vtk_is_dirty() {
                gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
                vtk_paint();
                gl.bind_framebuffer(glow::FRAMEBUFFER, None);
            }
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn texture_id(&mut self, frame: &mut eframe::Frame) -> egui::TextureId {
        // We are handing over ownership of the texture so should take care not to delete it
        *self
            .egui_texture_id
            .get_or_insert(frame.register_native_glow_texture(self.texture))
    }

    pub fn show(&mut self, ui: &mut egui::Ui, vtk_img: egui::Image) {
        let response = ui.add(vtk_img.sense(egui::Sense::all()));

        let current_size = response.rect.size();
        let width = current_size.x as i32;
        let height = current_size.y as i32;

        if width != self.width || height != self.height {
            println!("Updating size: {:#?}", current_size);

            unsafe {
                vtk_set_size(width, height);
            }
            self.width = width;
            self.height = height;
        }

        if response.hovered() {
            if let Some(pos) = ui.input(|i| i.pointer.latest_pos()) {
                let image_rect = response.rect;
                let relative_pos = pos - image_rect.min;
                let x = relative_pos.x as i32;
                let y = relative_pos.y as i32;

                unsafe {
                    vtk_mouse_move(x, y);
                }
            }

            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y as i32);
            if scroll_delta != 0 {
                unsafe {
                    vtk_mouse_wheel(scroll_delta);
                }
            }

            let (primary, secondary, middle) = ui.input(|i| {
                (
                    i.pointer.primary_down(),
                    i.pointer.secondary_down(),
                    i.pointer.middle_down(),
                )
            });

            unsafe { vtk_update_mouse_down(primary, secondary, middle) };
        }
    }
}
