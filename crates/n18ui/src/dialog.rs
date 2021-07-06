use gtk::prelude::*;
use std::collections::HashMap;

use n18game::{Company, Game};
use n18route::{Train, Trains};

/// A dialog for selecting a company.
pub struct CompanyDialog<'a> {
    pub dialog: gtk::Dialog,
    companies: &'a [&'a Company],
    combo: gtk::ComboBoxText,
}

impl<'a> CompanyDialog<'a> {
    pub fn new(
        parent: &gtk::ApplicationWindow,
        companies: &'a [&'a Company],
        title: &'a str,
    ) -> Self {
        let buttons = [
            ("OK", gtk::ResponseType::Accept),
            // ("Cancel", gtk::ResponseType::Reject),
        ];
        let flags = gtk::DialogFlags::all();
        let dialog = gtk::Dialog::with_buttons(
            Some(&title),
            Some(parent),
            flags,
            &buttons,
        );

        let padding = 4;
        let content = dialog.get_content_area();

        let combo = gtk::ComboBoxText::new();
        companies
            .iter()
            .for_each(|c| combo.append(Some(&c.abbrev), &c.full_name));
        combo.set_active_id(companies.get(0).map(|c| c.abbrev.as_str()));
        content.pack_start(&combo, true, false, padding);
        dialog.show_all();

        CompanyDialog {
            dialog,
            companies,
            combo,
        }
    }

    fn get_selected_ix(&self) -> Option<usize> {
        self.combo.get_active().and_then(|ix| {
            let ix = ix as usize;
            self.companies.get(ix).map(|_| ix)
        })
    }

    pub fn run(&self) -> Option<usize> {
        let response = self.dialog.run();
        self.dialog.hide();
        if response == gtk::ResponseType::Accept {
            self.get_selected_ix()
        } else {
            None
        }
    }
}

/// Display a company-selection dialog and return the selected company.
pub fn select_company<'a>(
    parent: &gtk::ApplicationWindow,
    companies: &'a [&'a Company],
) -> Option<usize> {
    let cd = CompanyDialog::new(parent, companies, "Select Company");
    cd.run()
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
        train_types: &'a Vec<&Train>,
        train_names: &'a HashMap<&Train, &str>,
        option_names: &'a Vec<&str>,
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
        let content = dialog.get_content_area();

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

    pub fn get_options(&self) -> Vec<bool> {
        self.options.iter().map(|cb| cb.get_active()).collect()
    }

    pub fn run(&self) -> Option<(Trains, Vec<bool>)> {
        let response = self.dialog.run();
        self.dialog.hide();
        if response == gtk::ResponseType::Accept {
            Some((self.get_trains(), self.get_options()))
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
    game: &Box<dyn Game>,
    name: &str,
) -> Option<(Trains, Vec<bool>)> {
    let train_types = game.train_types();
    let train_names: HashMap<_, &str> = train_types
        .iter()
        .map(|t| (*t, game.train_name(&t).unwrap()))
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
            Some(&title),
            Some(parent),
            flags,
            &buttons,
        );

        let padding = 4;
        let content = dialog.get_content_area();

        let combo = gtk::ComboBoxText::new();
        // phase_names.iter().for_each(|name| combo.append_text(name));
        let phase_names = phase_names.to_vec();
        phase_names
            .iter()
            .for_each(|name| combo.append(Some(name), name));
        combo.set_active_id(phase_names.get(current_phase).map(|s| *s));
        content.pack_start(&combo, true, false, padding);
        dialog.show_all();

        PhaseDialog {
            phase_names,
            combo,
            dialog,
        }
    }

    fn get_phase_ix(&self) -> Option<usize> {
        self.combo.get_active_text().and_then(|text| {
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
            self.get_phase_ix()
        } else {
            None
        }
    }
}

/// Display a phase-selection dialog and return the selected game phase.
pub fn select_phase(
    parent: &gtk::ApplicationWindow,
    game: &Box<dyn Game>,
) -> Option<usize> {
    let pd = PhaseDialog::new(
        parent,
        game.phase_names(),
        game.get_phase_ix(),
        "Select Game Phase",
    );
    pd.run()
}
