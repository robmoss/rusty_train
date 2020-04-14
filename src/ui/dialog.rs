use gtk::prelude::*;

use crate::route::train::{Train, Trains};

/// A dialog for selecting trains and options that provide route bonuses.
pub struct TrainDialog<'a> {
    trains: Vec<(&'a Train, gtk::SpinButton)>,
    options: Vec<gtk::CheckButton>,
    pub dialog: gtk::Dialog,
}

impl<'a> TrainDialog<'a> {
    pub fn new(
        parent: &gtk::ApplicationWindow,
        train_types: &'a Vec<Train>,
        name: &str,
    ) -> Self {
        let buttons = [
            ("OK", gtk::ResponseType::Accept),
            // ("Cancel", gtk::ResponseType::Reject),
        ];
        let flags = gtk::DialogFlags::all();
        let title = format!("{} Trains", name);
        let dialog = gtk::Dialog::new_with_buttons(
            Some(&title),
            Some(parent),
            flags,
            &buttons,
        );
        // TODO: the game should provide these route bonuses.
        let options = vec![
            gtk::CheckButton::new_with_label("Some Private Company"),
            gtk::CheckButton::new_with_label("Another Private Company"),
        ];

        let padding = 4;
        let content = dialog.get_content_area();
        let mut trains = Vec::with_capacity(train_types.len());
        train_types.iter().for_each(|train| {
            let row = add_spinner(train, &mut trains);
            content.pack_start(&row, true, true, padding)
        });
        options
            .iter()
            .for_each(|btn| content.pack_start(btn, true, true, padding));
        dialog.show_all();
        TrainDialog {
            trains,
            options,
            dialog,
        }
    }

    pub fn has_option(&self, ix: usize) -> Option<bool> {
        if ix < self.options.len() {
            Some(self.options[ix].get_active())
        } else {
            None
        }
    }

    pub fn get_trains(&self) -> Trains {
        let mut train_vec = vec![];
        self.trains.iter().for_each(|(train, spin)| {
            let num = spin.get_value() as usize;
            for _ in 0..num {
                train_vec.push(**train)
            }
        });
        train_vec.into()
    }

    pub fn run(&self) -> Option<Trains> {
        let response = self.dialog.run();
        self.dialog.hide();
        if response == gtk::ResponseType::Accept {
            // TODO: also return which route bonuses were selected.
            Some(self.get_trains())
        } else {
            None
        }
    }

    pub fn destroy(&self) {
        self.dialog.destroy();
    }
}

fn add_spinner<'a>(
    train: &'a Train,
    trains: &mut Vec<(&'a Train, gtk::SpinButton)>,
) -> gtk::Box {
    let spin = gtk::SpinButton::new_with_range(0.0, 9.0, 1.0);
    spin.set_digits(0);
    spin.set_numeric(true);
    spin.set_update_policy(gtk::SpinButtonUpdatePolicy::IfValid);
    let label = gtk::Label::new(Some(train.describe().as_str()));
    label.set_justify(gtk::Justification::Left);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    row.pack_start(&label, true, true, 0);
    row.pack_end(&spin, false, false, 0);
    trains.push((train, spin));
    row
}

/// Display a train-selection dialog and return the selected trains.
pub fn select<'a>(
    parent: &gtk::ApplicationWindow,
    train_types: &'a Vec<Train>,
    name: &str,
) -> Option<Trains> {
    let td = TrainDialog::new(parent, train_types, name);
    td.run()
}
