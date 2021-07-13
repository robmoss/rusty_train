use super::state::State;
use super::Action;

use cairo::{Context, Format, ImageSurface};
use gtk::{FileChooserExt, NativeDialogExt, WidgetExt};

use super::Content;
use n18game::GameState;

/// Prompt the user to select a file to which data will be saved.
pub fn save_file_dialog(
    window: &gtk::ApplicationWindow,
    title: &str,
    filters: &[&gtk::FileFilter],
    default_path: Option<&str>,
) -> Option<std::path::PathBuf> {
    let dialog = gtk::FileChooserNative::new(
        Some(title),
        Some(window),
        gtk::FileChooserAction::Save,
        Some("_Save"),
        Some("_Cancel"),
    );
    // Ask the user to confirm overwriting an existing file.
    dialog.set_do_overwrite_confirmation(true);
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
    let dialog = gtk::FileChooserNative::new(
        Some(title),
        Some(window),
        gtk::FileChooserAction::Open,
        Some("_Open"),
        Some("_Cancel"),
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
        dialog.destroy();
        None
    }
}

/// Prompt the user to save a screenshot.
pub fn save_screenshot<S: State + ?Sized>(
    state: &S,
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
    state.draw(content, &icx);
    let mut file = std::fs::File::create(dest_file)
        .unwrap_or_else(|_| panic!("Couldn't create '{}'", dest_str));
    surface.write_to_png(&mut file)?;
    Ok(Action::Redraw)
}

/// Prompt the user to save the current game state.
pub fn save_game(
    window: &gtk::ApplicationWindow,
    game_state: GameState,
) -> Result<Action, Box<dyn std::error::Error>> {
    let filter_game = gtk::FileFilter::new();
    filter_game.set_name(Some("Game files"));
    filter_game.add_pattern("*.game");
    let filter_all = gtk::FileFilter::new();
    filter_all.set_name(Some("All files"));
    filter_all.add_pattern("*");
    let filters = vec![&filter_game, &filter_all];
    let dest_file = save_file_dialog(window, "Save game", &filters, None);
    let dest_file = if let Some(dest) = dest_file {
        dest
    } else {
        return Ok(Action::None);
    };
    n18io::write_game_state(dest_file, game_state, true)?;
    Ok(Action::None)
}

/// Prompt the user to load a previously-saved game state.
pub fn load_game(
    window: &gtk::ApplicationWindow,
    content: &mut Content,
) -> Result<Action, Box<dyn std::error::Error>> {
    let filter_game = gtk::FileFilter::new();
    filter_game.set_name(Some("Game files"));
    filter_game.add_pattern("*.game");
    let filter_all = gtk::FileFilter::new();
    filter_all.set_name(Some("All files"));
    filter_all.add_pattern("*");
    let filters = vec![&filter_game, &filter_all];
    let dest_file = open_file_dialog(window, "Load game", &filters, None);
    let dest_file = if let Some(dest) = dest_file {
        dest
    } else {
        return Ok(Action::None);
    };
    let game_state = n18io::read_game_state(dest_file)?;
    let game_opt: Option<usize> =
        content.games.games.iter().enumerate().find_map(|(ix, g)| {
            if g.name() == game_state.game {
                Some(ix)
            } else {
                None
            }
        });
    if let Some(ix) = game_opt {
        content.games.set_active(ix);
        if let Some(new_map) =
            content.games.active_mut().load(&content.hex, game_state)
        {
            content.map = new_map;
            Ok(Action::ResetGame)
        } else {
            Ok(Action::None)
        }
    } else {
        Ok(Action::None)
    }
}
