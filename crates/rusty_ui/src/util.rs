use super::state::State;
use super::Action;

use cairo::{Context, Format, ImageSurface};
use gtk::{DialogExt, FileChooserExt, GtkWindowExt, WidgetExt};

use super::Content;
use rusty_hex::Hex;
use rusty_map::{HexAddress, HexIter, Map};

/// Prompt the user to select a file to which data will be saved.
pub fn save_file_dialog(
    window: &gtk::ApplicationWindow,
    title: &str,
    filters: &[&gtk::FileFilter],
    default_path: Option<&str>,
) -> Option<std::path::PathBuf> {
    let dialog = gtk::FileChooserDialog::with_buttons(
        Some(title),
        Some(window),
        gtk::FileChooserAction::Save,
        &[
            ("_Cancel", gtk::ResponseType::Cancel),
            ("_Save", gtk::ResponseType::Accept),
        ],
    );
    for filter in filters {
        dialog.add_filter(filter)
    }
    if let Some(path) = default_path {
        dialog.set_current_name(path);
    }
    let response = dialog.run();
    if response == gtk::ResponseType::Accept {
        let dest = dialog.get_filename().expect("Couldn't get filename");
        dialog.destroy();
        Some(dest)
    } else {
        dialog.close();
        dialog.destroy();
        None
    }
}

/// Prompt the user to select a file from which data will be read.
pub fn open_file_dialog(
    window: &gtk::ApplicationWindow,
    title: &str,
    filters: &[&gtk::FileFilter],
    default_path: Option<&str>,
) -> Option<std::path::PathBuf> {
    let dialog = gtk::FileChooserDialog::with_buttons(
        Some(title),
        Some(window),
        gtk::FileChooserAction::Open,
        &[
            ("_Cancel", gtk::ResponseType::Cancel),
            ("_Open", gtk::ResponseType::Accept),
        ],
    );
    for filter in filters {
        dialog.add_filter(filter)
    }
    if let Some(path) = default_path {
        dialog.set_current_name(path);
    }
    let response = dialog.run();
    if response == gtk::ResponseType::Accept {
        let dest = dialog.get_filename().expect("Couldn't get filename");
        dialog.destroy();
        Some(dest)
    } else {
        dialog.close();
        dialog.destroy();
        None
    }
}

/// Prompt the user to save a screenshot.
pub fn save_screenshot<S: State + ?Sized>(
    state: &Box<S>,
    window: &gtk::ApplicationWindow,
    area: &gtk::DrawingArea,
    content: &Content,
) -> Result<Action, Box<dyn std::error::Error>> {
    let filter_png = gtk::FileFilter::new();
    filter_png.set_name(Some("PNG images"));
    filter_png.add_mime_type("image/png");
    let filter_all = gtk::FileFilter::new();
    filter_all.set_name(Some("All files"));
    filter_all.add_pattern("*");
    let filters = vec![&filter_png, &filter_all];
    // Suggest a filename that contains the current date and time.
    let now = chrono::Local::now();
    let default_dest =
        now.format("screenshot-%Y-%m-%d-%H%M%S.png").to_string();
    let dest_file = save_file_dialog(
        window,
        "Save screenshot",
        &filters,
        Some(&default_dest),
    );
    let dest_file = if let Some(dest) = dest_file {
        dest
    } else {
        return Ok(Action::None);
    };
    let dest_str = dest_file.to_string_lossy().into_owned();
    // Use the same dimensions as the current drawing area.
    let width = area.get_allocated_width();
    let height = area.get_allocated_height();
    let surface = ImageSurface::create(Format::ARgb32, width, height)
        .expect("Can't create surface");
    let icx = Context::new(&surface);
    // Fill the image with a white background.
    icx.set_source_rgb(1.0, 1.0, 1.0);
    icx.paint();
    // Then draw the current map content.
    state.draw(content, width, height, &icx);
    let mut file = std::fs::File::create(dest_file)
        .expect(&format!("Couldn't create '{}'", dest_str));
    surface.write_to_png(&mut file)?;
    Ok(Action::Redraw)
}

/// Prompt the user to save the current map state.
pub fn save_map(
    window: &gtk::ApplicationWindow,
    map: &mut Map,
) -> Result<Action, Box<dyn std::error::Error>> {
    let filter_map = gtk::FileFilter::new();
    filter_map.set_name(Some("Map files"));
    filter_map.add_pattern("*.map");
    let filter_all = gtk::FileFilter::new();
    filter_all.set_name(Some("All files"));
    filter_all.add_pattern("*");
    let filters = vec![&filter_map, &filter_all];
    let dest_file =
        save_file_dialog(window, "Save map state", &filters, None);
    let dest_file = if let Some(dest) = dest_file {
        dest
    } else {
        return Ok(Action::None);
    };
    let descr: rusty_map::descr::Descr = (&*map).into();
    rusty_io::write_map_descr(dest_file, &descr, true)?;
    Ok(Action::None)
}

/// Prompt the user to load a previously-saved map state.
pub fn load_map(
    window: &gtk::ApplicationWindow,
    map: &mut Map,
) -> Result<Action, Box<dyn std::error::Error>> {
    let filter_map = gtk::FileFilter::new();
    filter_map.set_name(Some("Map files"));
    filter_map.add_pattern("*.map");
    let filter_all = gtk::FileFilter::new();
    filter_all.set_name(Some("All files"));
    filter_all.add_pattern("*");
    let filters = vec![&filter_map, &filter_all];
    let dest_file =
        open_file_dialog(window, "Load map state", &filters, None);
    let dest_file = if let Some(dest) = dest_file {
        dest
    } else {
        return Ok(Action::None);
    };
    let descr = rusty_io::read_map_descr(dest_file)?;
    descr.update_map(map);
    Ok(Action::Redraw)
}

pub fn draw_hex_backgrounds(
    hex: &Hex,
    ctx: &Context,
    mut hex_iter: &mut HexIter<'_>,
) {
    hex_iter.restart();
    for _ in &mut hex_iter {
        // Draw a thick black border on all hexes.
        // This will give map edges a clear border.
        hex.define_boundary(ctx);
        ctx.set_source_rgb(0.741, 0.86, 0.741);
        ctx.fill();
    }
    hex_iter.restart();
    for _ in &mut hex_iter {
        hex.define_boundary(ctx);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.set_line_width(hex.max_d * 0.05);
        ctx.stroke();
    }

    hex_iter.restart();
}

pub fn draw_empty_hex(hex: &Hex, ctx: &Context) {
    hex.define_boundary(ctx);
    ctx.set_source_rgb(0.741, 0.86, 0.741);
    ctx.fill();
}

pub fn outline_empty_hexes(
    hex: &Hex,
    ctx: &Context,
    mut hex_iter: &mut HexIter<'_>,
) {
    // Draw a thin grey border around empty hexes.
    hex_iter.restart();
    for (_addr, tile_opt) in &mut hex_iter {
        if tile_opt.is_none() {
            ctx.set_source_rgb(0.7, 0.7, 0.7);
            hex.define_boundary(ctx);
            ctx.set_line_width(hex.max_d * 0.01);
            ctx.stroke();
        }
    }

    hex_iter.restart();
}

pub fn highlight_active_hex(
    hex: &Hex,
    ctx: &Context,
    mut hex_iter: &mut HexIter<'_>,
    active_hex: &Option<HexAddress>,
    r: f64,
    g: f64,
    b: f64,
) {
    hex_iter.restart();
    for (addr, _tile_opt) in &mut hex_iter {
        if active_hex == &Some(addr) {
            // Draw the active hex with a coloured border.
            ctx.set_source_rgb(r, g, b);
            ctx.set_line_width(hex.max_d * 0.02);
            hex.define_boundary(ctx);
            ctx.stroke();
        } else {
            // Cover all other tiles with a partially-transparent layer.
            ctx.set_source_rgba(1.0, 1.0, 1.0, 0.25);
            hex.define_boundary(ctx);
            ctx.fill();
        }
    }

    hex_iter.restart();
}
