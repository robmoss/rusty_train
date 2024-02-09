//! Provides a user interface controller for GTK 4.

use gtk::prelude::*;
use gtk4 as gtk;
use std::collections::BTreeMap;

use n18game::{DividendOptions, Game};
use n18route::{Train, Trains};

use crate::{PingDest, PingSender, UiController};

/// Prompts the user to select one string from `items` and provides the
/// selected string (if any) to `callback`.
pub fn select_string<F>(
    parent: &gtk::Window,
    title: &str,
    items: &[&str],
    callback: F,
) where
    F: Fn(Option<String>) + 'static,
{
    let strs: Vec<String> = items.iter().map(|s| s.to_string()).collect();
    select_index(parent, title, items, move |ix_opt| {
        callback(ix_opt.map(|ix| strs[ix].clone()))
    })
}

/// Prompts the user to select one item from `items` and provides the index of
/// the selected item (if any) to `callback`.
pub fn select_index<F>(
    parent: &gtk::Window,
    title: &str,
    items: &[&str],
    callback: F,
) where
    F: Fn(Option<usize>) + 'static,
{
    let buttons = [
        ("OK", gtk::ResponseType::Accept),
        // ("Cancel", gtk::ResponseType::Cancel),
    ];
    let flags = gtk::DialogFlags::all();
    let dialog =
        gtk::Dialog::with_buttons(Some(title), Some(parent), flags, &buttons);

    let padding = 4;
    let content = dialog.content_area();

    let title_label = gtk::Label::new(Some(title));

    // Display the companies as a list.
    let list = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::Browse)
        .activate_on_single_click(false)
        .margin_top(padding)
        .margin_bottom(padding)
        .margin_start(padding)
        .margin_end(padding)
        .build();

    items.iter().for_each(|item| {
        let item_label = gtk::Label::new(Some(item));
        list.append(&item_label);
    });

    // Select the first item.
    list.select_row(list.row_at_index(0).as_ref());

    // Make activating a row pick the active item and close the dialog.
    let dlg = dialog.clone();
    list.connect_row_activated(move |_, _| {
        dlg.response(gtk::ResponseType::Accept)
    });

    title_label.set_margin_top(padding);
    title_label.set_margin_bottom(padding);
    title_label.set_margin_start(padding);
    title_label.set_margin_end(padding);
    list.set_margin_top(padding);
    list.set_margin_bottom(padding);
    list.set_margin_start(padding);
    list.set_margin_end(padding);

    content.set_spacing(padding);
    content.set_orientation(gtk::Orientation::Vertical);
    content.append(&title_label);
    content.append(&list);

    dialog.connect_response(move |dlg, response| {
        dlg.hide();
        let ix = if response == gtk::ResponseType::Accept {
            list.selected_row().map(|row| row.index() as usize)
        } else {
            None
        };
        callback(ix)
    });
    dialog.show();
}

/// Prompts the user to select the trains and operating bonuses for a company,
/// and provides these details (if any) to `callback`.
#[allow(clippy::needless_collect)]
pub fn select_trains<F>(
    parent: &gtk::Window,
    game: &dyn Game,
    name: &str,
    callback: F,
) where
    F: Fn(Option<(Trains, Vec<bool>)>) + 'static,
{
    let train_types = game.train_types();
    let train_names: BTreeMap<_, &str> = train_types
        .iter()
        .map(|t| (*t, game.train_name(t).unwrap()))
        .collect();
    let option_names = game.bonus_options();

    let buttons = [
        ("OK", gtk::ResponseType::Accept),
        // ("Cancel", gtk::ResponseType::Reject),
    ];
    let flags = gtk::DialogFlags::all();
    let title = format!("{} Trains", name);
    let dialog = gtk::Dialog::with_buttons(
        Some(&title),
        Some(parent),
        flags,
        &buttons,
    );
    let options: Vec<_> = option_names
        .iter()
        .map(|name| gtk::CheckButton::with_label(name))
        .collect();

    let padding = 4;
    let spacing = 8;

    // NOTE: pack the train list on the left, and options on the right.
    let content = dialog.content_area();
    content.set_orientation(gtk::Orientation::Horizontal);
    content.set_spacing(padding);

    let train_col = gtk::Box::new(gtk::Orientation::Vertical, spacing);
    let option_col = gtk::Box::new(gtk::Orientation::Vertical, spacing);
    let option_box = gtk::Box::new(gtk::Orientation::Vertical, spacing);
    train_col.set_spacing(padding);
    option_col.set_spacing(padding);
    option_box.set_spacing(padding);
    train_col.set_margin_bottom(padding);
    train_col.set_margin_top(padding);
    train_col.set_margin_start(padding);
    train_col.set_margin_end(padding);
    option_col.set_margin_bottom(padding);
    option_col.set_margin_top(padding);
    option_col.set_margin_start(padding);
    option_col.set_margin_end(padding);
    option_box.set_margin_bottom(padding);
    option_box.set_margin_top(padding);
    option_box.set_margin_start(padding);
    option_box.set_margin_end(padding);
    content.append(&train_col);
    content.append(&option_col);
    option_box.set_valign(gtk::Align::Center);
    option_box.set_vexpand(true);
    option_col.append(&option_box);

    let mut trains = Vec::with_capacity(train_types.len());
    train_types.iter().for_each(|train| {
        let row =
            add_spinner(train, train_names.get(train).unwrap(), &mut trains);
        train_col.append(&row)
    });
    options.iter().for_each(|btn| {
        btn.set_margin_bottom(padding);
        btn.set_margin_top(padding);
        btn.set_margin_start(padding);
        btn.set_margin_end(padding);
        option_box.append(btn)
    });

    // NOTE: we need to collect these values into a vector so that they can
    // be owned by the closure below.
    // This produces a `clippy::needless_collect` warning unless we disable
    // the lint for this function.
    let trains: Vec<(Train, _)> = trains
        .into_iter()
        .map(|(&train, spin)| (train, spin))
        .collect();
    dialog.connect_response(move |dlg, response| {
        dlg.hide();
        if response == gtk::ResponseType::Accept {
            let mut train_vec = vec![];
            trains.iter().for_each(|(train, spin)| {
                let num = spin.value() as usize;
                for _ in 0..num {
                    train_vec.push(*train)
                }
            });
            let trains: Trains = train_vec.into();
            let opts = options.iter().map(|cb| cb.is_active()).collect();
            callback(Some((trains, opts)))
        } else {
            callback(None)
        }
    });
    dialog.show();
}

/// Prompts the user to select a game phase, and provides this phase (if any)
/// to `callback`.
pub fn select_phase<F>(parent: &gtk::Window, game: &dyn Game, callback: F)
where
    F: Fn(Option<usize>) + 'static,
{
    let title = "Select Game Phase";
    let phase_names = game.phase_names().to_vec();
    let current_phase = game.phase_ix();

    let buttons = [
        ("OK", gtk::ResponseType::Accept),
        // ("Cancel", gtk::ResponseType::Cancel),
    ];
    let flags = gtk::DialogFlags::all();
    let dialog =
        gtk::Dialog::with_buttons(Some(title), Some(parent), flags, &buttons);

    let padding = 4;
    let content = dialog.content_area();

    let combo = gtk::ComboBoxText::new();
    phase_names
        .iter()
        .for_each(|name| combo.append(Some(name), name));
    combo.set_active_id(phase_names.get(current_phase).copied());
    combo.set_margin_bottom(padding);
    combo.set_margin_top(padding);
    combo.set_margin_start(padding);
    combo.set_margin_end(padding);
    content.append(&combo);

    let phase_names: Vec<String> =
        phase_names.iter().map(|s| s.to_string()).collect();
    dialog.connect_response(move |dlg, response| {
        dlg.hide();
        if response == gtk::ResponseType::Accept {
            let ix = combo.active_text().and_then(|text| {
                let text = text.as_str();
                phase_names
                    .iter()
                    .enumerate()
                    .find(|(_ix, name)| text == *name)
                    .map(|(ix, _name)| ix)
            });
            callback(ix)
        } else {
            callback(None)
        }
    });
    dialog.show();
}

/// Returns a `gtk::Box` that contains a `gtk::SpinButton` and a `gtk::Label`,
/// and adds `(train, spin_button)` to the vector `trains`.
fn add_spinner<'a>(
    train: &'a Train,
    name: &'a str,
    trains: &mut Vec<(&'a Train, gtk::SpinButton)>,
) -> gtk::Box {
    let spin = gtk::SpinButton::with_range(0.0, 9.0, 1.0);
    spin.set_digits(0);
    spin.set_numeric(true);
    spin.set_update_policy(gtk::SpinButtonUpdatePolicy::IfValid);
    let label = gtk::Label::new(Some(name));
    label.set_justify(gtk::Justification::Left);
    label.set_hexpand(true);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 16);
    row.append(&label);
    row.append(&spin);
    let padding = 4;
    row.set_margin_bottom(padding);
    row.set_margin_top(padding);
    row.set_margin_start(padding);
    row.set_margin_end(padding);
    trains.push((train, spin));
    row
}

/// Prompts the user to select a file to which data will be saved, and
/// provides the selected filename (if any) to `callback`.
pub fn select_file_save<F>(
    window: &gtk::Window,
    title: &str,
    filters: &[gtk::FileFilter],
    default_path: Option<&str>,
    callback: F,
) where
    F: Fn(Option<std::path::PathBuf>) + 'static,
{
    let dialog = gtk::FileChooserNative::builder()
        .accept_label("_Save")
        .cancel_label("_Cancel")
        .modal(true)
        .title(title)
        .transient_for(window)
        .action(gtk::FileChooserAction::Save)
        .build();

    for filter in filters {
        dialog.add_filter(filter)
    }
    if let Some(path) = default_path {
        dialog.set_current_name(path);
    }

    // NOTE: we need to clone `dialog` so that we can keep it alive.
    // Otherwise it will be dropped when this function returns, and the dialog
    // will be destroyed.
    // This is because native dialogs are **not** GTK widgets, and so GTK does
    // not manage their life-cycle.
    let live_dialog = dialog.clone();
    dialog.connect_response(move |_dlg, response| {
        live_dialog.hide();
        if response == gtk::ResponseType::Accept {
            let dest = live_dialog
                .file()
                .expect("Couldn't get file")
                .path()
                .expect("Couldn't get filename");
            callback(Some(dest))
        } else {
            callback(None)
        }
    });

    dialog.show();
}

/// Prompts the user to select a file from which data will be read, and
/// provides the selected filename (if any) to `callback`.
pub fn select_file_load<F>(
    window: &gtk::Window,
    title: &str,
    filters: &[gtk::FileFilter],
    default_path: Option<&str>,
    callback: F,
) where
    F: Fn(Option<std::path::PathBuf>) + 'static,
{
    let dialog = gtk::FileChooserNative::builder()
        .accept_label("_Save")
        .cancel_label("_Cancel")
        .modal(true)
        .title(title)
        .transient_for(window)
        .action(gtk::FileChooserAction::Open)
        .build();
    for filter in filters {
        dialog.add_filter(filter)
    }
    if let Some(path) = default_path {
        dialog.set_current_name(path);
    }

    // NOTE: we need to clone `dialog` so that we can keep it alive.
    // Otherwise it will be dropped when this function returns, and the dialog
    // will be destroyed.
    // This is because native dialogs are **not** GTK widgets, and so GTK does
    // not manage their life-cycle.
    let live_dialog = dialog.clone();
    dialog.connect_response(move |_dlg, response| {
        live_dialog.hide();
        if response == gtk::ResponseType::Accept {
            let dest = live_dialog
                .file()
                .expect("Couldn't get file")
                .path()
                .expect("Couldn't get filename");
            callback(Some(dest))
        } else {
            callback(None)
        }
    });

    dialog.set_modal(true);
    dialog.show();
}

/// Returns the default file filters when loading/saving an image.
pub fn image_file_filters() -> Vec<gtk::FileFilter> {
    let filter_png = gtk::FileFilter::new();
    filter_png.set_name(Some("PNG images"));
    filter_png.add_mime_type("image/png");
    let filter_all = gtk::FileFilter::new();
    filter_all.set_name(Some("All files"));
    filter_all.add_pattern("*");
    vec![filter_png, filter_all]
}

/// Returns the default file filters when loading/saving a game state.
pub fn game_file_filters() -> Vec<gtk::FileFilter> {
    let filter_game = gtk::FileFilter::new();
    filter_game.set_name(Some("Game files"));
    filter_game.add_pattern("*.game");
    let filter_all = gtk::FileFilter::new();
    filter_all.set_name(Some("All files"));
    filter_all.add_pattern("*");
    vec![filter_game, filter_all]
}

/// A user interface controller for GTK, which draws the game map on a
/// `DrawingArea` widget.
pub struct GtkController {
    window: gtk::Window,
    draw_area: gtk::DrawingArea,
    ping_tx: async_channel::Sender<PingDest>,
}

impl GtkController {
    pub fn new(
        window: gtk::Window,
        draw_area: gtk::DrawingArea,
        ping_tx: async_channel::Sender<PingDest>,
    ) -> Self {
        GtkController {
            window,
            draw_area,
            ping_tx,
        }
    }
}

impl UiController for GtkController {
    fn quit(&mut self) {
        self.window.close();
    }

    fn redraw(&self) {
        self.draw_area.queue_draw();
    }

    fn set_window_title(&mut self, title: &str) {
        self.window.set_title(Some(title))
    }

    fn window_title(&self) -> Option<String> {
        self.window.title().map(|gs| gs.to_string())
    }

    fn resize(&mut self, width: i32, height: i32) {
        self.draw_area.set_size_request(width, height);
    }

    fn ping_tx(&self) -> PingSender {
        PingSender::Async(self.ping_tx.clone())
    }

    fn select_string<F>(&mut self, title: &str, strings: &[&str], callback: F)
    where
        F: Fn(Option<String>) + 'static,
    {
        select_string(&self.window, title, strings, callback)
    }

    fn select_index<F>(&mut self, title: &str, strings: &[&str], callback: F)
    where
        F: Fn(Option<usize>) + 'static,
    {
        select_index(&self.window, title, strings, callback)
    }

    fn select_trains<F>(&mut self, game: &dyn Game, title: &str, callback: F)
    where
        Self: Sized,
        F: Fn(Option<(Trains, Vec<bool>)>) + 'static,
    {
        select_trains(&self.window, game, title, callback)
    }

    fn select_phase<F>(&mut self, game: &dyn Game, callback: F)
    where
        Self: Sized,
        F: Fn(Option<usize>) + 'static,
    {
        select_phase(&self.window, game, callback)
    }

    fn select_screenshot_save<F>(
        &mut self,
        title: &str,
        default_path: Option<&str>,
        callback: F,
    ) where
        Self: Sized,
        F: Fn(Option<std::path::PathBuf>) + 'static,
    {
        let filters = image_file_filters();
        select_file_save(
            &self.window,
            title,
            &filters,
            default_path,
            callback,
        )
    }

    fn select_game_save<F>(
        &mut self,
        title: &str,
        default_path: Option<&str>,
        callback: F,
    ) where
        Self: Sized,
        F: Fn(Option<std::path::PathBuf>) + 'static,
    {
        let filters = game_file_filters();
        select_file_save(
            &self.window,
            title,
            &filters,
            default_path,
            callback,
        )
    }

    fn select_game_load<F>(
        &mut self,
        title: &str,
        default_path: Option<&str>,
        callback: F,
    ) where
        Self: Sized,
        F: Fn(Option<std::path::PathBuf>) + 'static,
    {
        let filters = game_file_filters();
        select_file_load(
            &self.window,
            title,
            &filters,
            default_path,
            callback,
        )
    }

    fn show_dividends<F>(
        &mut self,
        abbrev: &str,
        revenue: usize,
        options: DividendOptions,
        callback: F,
    ) where
        Self: Sized,
        F: Fn() + 'static,
    {
        let title = format!("{} dividends", abbrev);
        let buttons = [("OK", gtk::ResponseType::Accept)];
        let flags = gtk::DialogFlags::all();
        let dialog = gtk::Dialog::with_buttons(
            Some(&title),
            Some(&self.window),
            flags,
            &buttons,
        );

        let dividends = options.dividends(revenue);
        let n = options.share_count;
        let any_withheld = dividends.iter().any(|d| d.withheld.is_some());

        let grid = gtk::Grid::builder()
            .hexpand(true)
            .vexpand(true)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .column_spacing(24)
            .row_spacing(8)
            .row_homogeneous(true)
            .margin_bottom(16)
            .margin_top(16)
            .margin_start(16)
            .margin_end(16)
            .build();

        // Add a title label to a (column, row) cell.
        let add_title = |column: usize, row: usize, text: &str| {
            let label = gtk::Label::builder()
                .use_markup(true)
                .selectable(false)
                .label(format!("<b>{}</b>", text))
                .hexpand(true)
                .vexpand(true)
                .halign(gtk::Align::Center)
                .build();
            grid.attach(&label, column as i32, row as i32, 1, 1);
        };

        // Add a normal label to a (column, row) cell.
        let add_label = |column: usize, row: usize, text: &str| {
            let label = gtk::Label::builder()
                .use_markup(false)
                .selectable(false)
                .label(text)
                .hexpand(true)
                .vexpand(true)
                .halign(gtk::Align::End)
                .build();
            grid.attach(&label, column as i32, row as i32, 1, 1);
        };

        // Create a column for each number of shares.
        add_title(0, 0, "# Shares");
        for i in 1..=n {
            add_label(0, i, &format!("{}", i));
        }

        // Create a column for each kind of dividend.
        for (ix, div) in dividends.iter().enumerate() {
            let column = ix + 1;
            add_title(column, 0, &format!("{}", div.kind));
            for (p_ix, amount) in div.share_payments.iter().enumerate() {
                let text = format!("${}", amount);
                add_label(column, p_ix + 1, &text);
            }
        }

        // Create an extra row for any withheld amounts.
        if any_withheld {
            let row = n + 1;
            add_label(0, row, "");
            add_title(0, row + 1, "Withheld:");
            for (ix, div) in dividends.iter().enumerate() {
                let column = ix + 1;
                if let Some(amount) = div.withheld {
                    let text = format!("${}", amount);
                    add_label(column, row + 1, &text);
                }
            }
        }

        let content = dialog.content_area();
        content.append(&grid);

        dialog.connect_response(move |dlg, _response| {
            dlg.hide();
            callback()
        });

        dialog.show();
    }
}
