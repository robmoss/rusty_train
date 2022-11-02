use log::info;
use std::sync::{Arc, RwLock};

use crate::{Assets, State};
use n18hex::Hex;

/// Returns the ink bounding box `(x0, y0, width, height)` for the provided
/// state.
pub fn ink_extents(state: &State, assets: &Assets) -> (f64, f64, f64, f64) {
    let surf = cairo::RecordingSurface::create(cairo::Content::Color, None)
        .expect("Could not create RecordingSurface");

    let ctx =
        cairo::Context::new(&surf).expect("Could not create cairo::Context");
    state.draw(assets, &ctx);
    // Note: (x0, y0, width, height)
    surf.ink_extents()
}

/// Returns the ink bounding box `(x0, y0, width, height)` for the provided
/// state, for the specified maximal hex diameter `hex_d`.
pub fn ink_extents_with_hex(
    state: &State,
    assets: &mut Assets,
    hex_d: f64,
) -> (f64, f64, f64, f64) {
    let mut new_hex = Hex::new(hex_d);
    std::mem::swap(&mut new_hex, &mut assets.hex);
    let exts = ink_extents(state, assets);
    std::mem::swap(&mut new_hex, &mut assets.hex);
    exts
}

/// Returns the surface dimensions required to draw the provided state.
pub fn required_dims(state: &State, assets: &Assets) -> (i32, i32) {
    let exts = ink_extents(state, assets);
    let want_width = (exts.2 + 2.0 * exts.0) as i32;
    let want_height = (exts.3 + 2.0 * exts.1) as i32;
    (want_width, want_height)
}

/// Returns the surface dimensions required to draw the provided state at the
/// maximum zoom level.
pub fn max_surface_dims(
    state: &State,
    assets: &mut Assets,
    hex_d: f64,
) -> (i32, i32) {
    // NOTE: this is the upper limit on the maximum hex size.
    let exts = ink_extents_with_hex(state, assets, hex_d);
    let want_width = (exts.2 + 2.0 * exts.0) as i32;
    let want_height = (exts.3 + 2.0 * exts.1) as i32;
    (want_width, want_height)
}

pub struct Canvas {
    // NOTE: we need to share the surface with the main event loop and the UI.
    surface: Arc<RwLock<cairo::ImageSurface>>,
    context: cairo::Context,
    width: i32,
    height: i32,
}

impl Canvas {
    pub fn new(width: i32, height: i32) -> Self {
        let surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, width, height)
                .expect("Could not create ImageSurface");
        let context = cairo::Context::new(&surface)
            .expect("Could not create cairo::Context");
        let surface = Arc::new(RwLock::new(surface));
        Canvas {
            surface,
            context,
            width,
            height,
        }
    }

    pub fn surface(&self) -> Arc<RwLock<cairo::ImageSurface>> {
        Arc::clone(&self.surface)
    }

    pub fn copy_surface(&self) -> cairo::ImageSurface {
        let source = self
            .surface
            .read()
            .expect("Could not access drawing surface");
        let new_surface = cairo::ImageSurface::create(
            source.format(),
            source.width(),
            source.height(),
        )
        .expect("Could not create image surface");
        let ctx = cairo::Context::new(&new_surface)
            .expect("Could not create image context");
        ctx.set_source_surface(&*source, 0.0, 0.0).unwrap();
        ctx.set_operator(cairo::Operator::Source);
        ctx.paint().unwrap();
        new_surface
    }

    pub fn copy_ink(
        &self,
        state: &State,
        assets: &Assets,
    ) -> cairo::ImageSurface {
        let source = self
            .surface
            .read()
            .expect("Could not access drawing surface");
        let (width, height) = required_dims(state, assets);
        let new_surface =
            cairo::ImageSurface::create(source.format(), width, height)
                .expect("Could not create image surface");
        let ctx = cairo::Context::new(&new_surface)
            .expect("Could not create image context");
        ctx.set_source_surface(&*source, 0.0, 0.0).unwrap();
        ctx.set_operator(cairo::Operator::Source);
        ctx.paint().unwrap();
        new_surface
    }

    pub fn copy_ink_with_margin(
        &self,
        state: &State,
        assets: &Assets,
        margin: i32,
    ) -> cairo::ImageSurface {
        let source = self
            .surface
            .read()
            .expect("Could not access drawing surface");
        let exts = ink_extents(state, assets);
        let width = (exts.2 + 2.0 * margin as f64) as i32;
        let height = (exts.3 + 2.0 * margin as f64) as i32;
        let x0 = margin as f64 - exts.0;
        let y0 = margin as f64 - exts.1;
        let new_surface =
            cairo::ImageSurface::create(source.format(), width, height)
                .expect("Could not create image surface");
        let ctx = cairo::Context::new(&new_surface)
            .expect("Could not create image context");
        ctx.set_source_surface(&*source, x0, y0).unwrap();
        ctx.set_operator(cairo::Operator::Source);
        ctx.paint().unwrap();
        new_surface
    }

    pub fn context(&self) -> &cairo::Context {
        &self.context
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn resize(&mut self, new_width: i32, new_height: i32) {
        info!(
            "Resizing image surface from ({}, {}) to ({}, {})",
            self.width, self.height, new_width, new_height
        );
        let surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            new_width,
            new_height,
        )
        .expect("Could not create ImageSurface");
        self.context = cairo::Context::new(&surface)
            .expect("Could not create cairo::Context");
        let mut surf_ref = self
            .surface
            .write()
            .expect("Could not modify drawing surface");
        *surf_ref = surface;
        self.width = new_width;
        self.height = new_height;
    }
}
