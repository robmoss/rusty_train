use gtk::prelude::*;
use std::collections::HashMap;

use n18game::Game;
use n18route::{Train, Trains};

/// A dialog for selecting one item from a list.
pub struct SelectItemDialog {
    pub dialog: gtk::Dialog,
    list: gtk::ListBox,
}

impl SelectItemDialog {
    pub fn new(
        parent: &gtk::ApplicationWindow,
        title: &str,
        items: &[&str],
    ) -> Self {
        let buttons = [
            ("OK", gtk::ResponseType::Accept),
            // ("Cancel", gtk::ResponseType::Reject),
        ];
        let flags = gtk::DialogFlags::all();
        let dialog = gtk::Dialog::with_buttons(
            Some(title),
            Some(parent),
            flags,
            &buttons,
        );

        let padding = 4;
        let content = dialog.content_area();

        let title_label = gtk::Label::new(Some(title));

        // Display the companies as a list.
        let list = gtk::ListBoxBuilder::new()
            .selection_mode(gtk::SelectionMode::Browse)
            .activate_on_single_click(false)
            .margin(4)
            .build();
        items.iter().for_each(|item| {
            let item_label = gtk::Label::new(Some(item));
            list.add(&item_label);
        });
        // Select the first item.
        list.select_row(list.row_at_index(0).as_ref());
        // Make activating a row pick the active item and close the dialog.
        let dlg = dialog.clone();
        list.connect_row_activated(move |_, _| {
            dlg.response(gtk::ResponseType::Accept)
        });
        content.pack_start(&title_label, true, false, padding);
        content.pack_start(&list, true, false, padding);
        dialog.show_all();

        SelectItemDialog { dialog, list }
    }

    fn selected_ix(&self) -> Option<usize> {
        self.list.selected_row().map(|row| row.index() as usize)
    }

    pub fn run(&self) -> Option<usize> {
        let response = self.dialog.run();
        self.dialog.hide();
        if response == gtk::ResponseType::Accept {
            self.selected_ix()
        } else {
            None
        }
    }
}

/// Display an item-selection dialog and return the selected item index.
pub fn select_item_index<'a>(
    parent: &gtk::ApplicationWindow,
    title: &str,
    items: &'a [&'a str],
) -> Option<usize> {
    if items.is_empty() {
        return None;
    }
    let dlg = SelectItemDialog::new(parent, title, items);
    dlg.run()
}

/// Display an item-selection dialog and return the selected item.
pub fn select_item<'a>(
    parent: &gtk::ApplicationWindow,
    title: &str,
    items: &'a [&'a str],
) -> Option<&'a str> {
    select_item_index(parent, title, items).map(|ix| items[ix])
}

/// A dialog for selecting trains and options that provide route bonuses.
pub struct TrainDialog<'a> {
    trains: Vec<(&'a Train, gtk::SpinButton)>,
    options: Vec<gtk::CheckButton>,
    pub dialog: gtk::Dialog,
}

impl<'a> TrainDialog<'a> {
    pub fn new(
        parent: &gtk::ApplicationWindow,
        train_types: &'a [&Train],
        train_names: &'a HashMap<&Train, &str>,
        option_names: &'a [&str],
        name: &str,
    ) -> Self {
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
        let content = dialog.content_area();

        // NOTE: pack the train list on the left, and options on the right.
        let train_col = gtk::Box::new(gtk::Orientation::Vertical, spacing);
        let option_col = gtk::Box::new(gtk::Orientation::Vertical, spacing);
        let option_box = gtk::Box::new(gtk::Orientation::Vertical, spacing);
        content.set_orientation(gtk::Orientation::Horizontal);
        content.pack_start(&train_col, true, true, padding);
        content.pack_start(&option_col, true, true, padding);
        option_col.pack_start(&option_box, true, false, padding);

        let mut trains = Vec::with_capacity(train_types.len());
        train_types.iter().for_each(|train| {
            let row = add_spinner(
                train,
                train_names.get(train).unwrap(),
                &mut trains,
            );
            train_col.pack_start(&row, false, false, padding)
        });
        options.iter().for_each(|btn| {
            option_box.pack_start(btn, false, false, padding)
        });
        dialog.show_all();
        TrainDialog {
            trains,
            options,
            dialog,
        }
    }

    pub fn has_option(&self, ix: usize) -> Option<bool> {
        if ix < self.options.len() {
            Some(self.options[ix].is_active())
        } else {
            None
        }
    }

    pub fn trains(&self) -> Trains {
        let mut train_vec = vec![];
        self.trains.iter().for_each(|(train, spin)| {
            let num = spin.value() as usize;
            for _ in 0..num {
                train_vec.push(**train)
            }
        });
        train_vec.into()
    }

    pub fn options(&self) -> Vec<bool> {
        self.options.iter().map(|cb| cb.is_active()).collect()
    }

    pub fn run(&self) -> Option<(Trains, Vec<bool>)> {
        let response = self.dialog.run();
        self.dialog.hide();
        if response == gtk::ResponseType::Accept {
            Some((self.trains(), self.options()))
        } else {
            None
        }
    }
}

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
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 16);
    row.pack_start(&label, true, true, 0);
    row.pack_end(&spin, false, false, 0);
    trains.push((train, spin));
    row
}

/// Display a train-selection dialog and return the selected trains and route
/// bonuses.
pub fn select_trains(
    parent: &gtk::ApplicationWindow,
    game: &dyn Game,
    name: &str,
) -> Option<(Trains, Vec<bool>)> {
    let train_types = game.train_types();
    let train_names: HashMap<_, &str> = train_types
        .iter()
        .map(|t| (*t, game.train_name(t).unwrap()))
        .collect();
    let option_names = game.bonus_options();
    let td = TrainDialog::new(
        parent,
        &train_types,
        &train_names,
        &option_names,
        name,
    );
    td.run()
}

/// A dialog for selecting the game phase.
pub struct PhaseDialog<'a> {
    phase_names: Vec<&'a str>,
    combo: gtk::ComboBoxText,
    pub dialog: gtk::Dialog,
}

impl<'a> PhaseDialog<'a> {
    pub fn new(
        parent: &gtk::ApplicationWindow,
        phase_names: &'a [&'a str],
        current_phase: usize,
        title: &str,
    ) -> Self {
        let buttons = [
            ("OK", gtk::ResponseType::Accept),
            // ("Cancel", gtk::ResponseType::Reject),
        ];
        let flags = gtk::DialogFlags::all();
        let dialog = gtk::Dialog::with_buttons(
            Some(title),
            Some(parent),
            flags,
            &buttons,
        );

        let padding = 4;
        let content = dialog.content_area();

        let combo = gtk::ComboBoxText::new();
        // phase_names.iter().for_each(|name| combo.append_text(name));
        let phase_names = phase_names.to_vec();
        phase_names
            .iter()
            .for_each(|name| combo.append(Some(name), name));
        combo.set_active_id(phase_names.get(current_phase).copied());
        content.pack_start(&combo, true, false, padding);
        dialog.show_all();

        PhaseDialog {
            phase_names,
            combo,
            dialog,
        }
    }

    fn phase_ix(&self) -> Option<usize> {
        self.combo.active_text().and_then(|text| {
            let text = text.as_str();
            self.phase_names
                .iter()
                .enumerate()
                .find(|(_ix, name)| &text == *name)
                .map(|(ix, _name)| ix)
        })
    }

    pub fn run(&self) -> Option<usize> {
        let response = self.dialog.run();
        self.dialog.hide();
        if response == gtk::ResponseType::Accept {
            self.phase_ix()
        } else {
            None
        }
    }
}

/// Display a phase-selection dialog and return the selected game phase.
pub fn select_phase(
    parent: &gtk::ApplicationWindow,
    game: &dyn Game,
) -> Option<usize> {
    let pd = PhaseDialog::new(
        parent,
        game.phase_names(),
        game.phase_ix(),
        "Select Game Phase",
    );
    pd.run()
}
