use cairo::{Context, Format, ImageSurface};
use gtk::{DialogExt, FileChooserExt, GtkWindowExt, Inhibit, WidgetExt};

use crate::hex::Hex;
use crate::map::{HexAddress, Map, RotateCW};

/// The actions that may be required when the UI state changes.
pub enum Action {
    /// No action required.
    None,
    /// Quit the application.
    Quit,
    /// Redraw the surface.
    Redraw,
}

/// The methods that are required in order to manipulate the user interface.
pub trait State {
    /// Draws the current state of the map.
    fn draw(
        &self,
        hex: &Hex,
        map: &Map,
        width: i32,
        height: i32,
        ctx: &Context,
    );

    /// Responds to a key being pressed, and returns the new state.
    fn key_press(
        self: Box<Self>,
        hex: &Hex,
        map: &mut Map,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action);

    /// Responds to a mouse button being clicked, and returns the new state.
    fn button_press(
        self: Box<Self>,
        hex: &Hex,
        map: &mut Map,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &gdk::EventButton,
    ) -> (Box<dyn State>, Inhibit, Action);
}

/// The default state: selecting a tile.
pub struct Default {
    active_hex: Option<HexAddress>,
}

/// Replacing one tile with another.
pub struct ReplaceTile {
    active_hex: HexAddress,
    candidates: Vec<usize>,
    selected: usize,
    show_original: bool,
    rotation: RotateCW,
}

impl Default {
    pub fn new(map: &Map) -> Self {
        Default {
            active_hex: map.default_hex(),
        }
    }
}

impl ReplaceTile {
    fn with_any(map: &Map, addr: HexAddress) -> Self {
        let candidates: Vec<usize> = (0..(map.tiles().len())).collect();
        ReplaceTile {
            active_hex: addr,
            candidates,
            selected: 0,
            show_original: false,
            rotation: RotateCW::Zero,
        }
    }

    fn with_candidates(addr: HexAddress, candidates: Vec<usize>) -> Self {
        ReplaceTile {
            active_hex: addr,
            candidates,
            selected: 0,
            show_original: false,
            rotation: RotateCW::Zero,
        }
    }
}

impl State for Default {
    fn draw(
        &self,
        hex: &Hex,
        map: &Map,
        _width: i32,
        _height: i32,
        ctx: &Context,
    ) {
        for (_addr, tile_opt) in map.hex_iter(hex, ctx) {
            if let Some((tile, tokens)) = tile_opt {
                // Draw the tile and any tokens.
                tile.draw(ctx, hex);
                for (tok, map_token) in tokens.iter() {
                    tile.define_tok_path(&tok, &hex, ctx);
                    map_token.draw_token(&hex, ctx);
                }
            } else {
                // Draw an empty hex.
                // TODO: draw "crosshairs" at hex intersections?
                ctx.set_source_rgb(0.7, 0.7, 0.7);
                hex.define_boundary(ctx);
                ctx.set_line_width(hex.max_d * 0.01);
                ctx.stroke();
            }
        }

        for (addr, _tile_opt) in map.hex_iter(hex, ctx) {
            if self.active_hex == Some(addr) {
                // Draw the active hex with a red border.
                ctx.set_source_rgb(0.7, 0.0, 0.0);
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
    }

    fn key_press(
        mut self: Box<Self>,
        hex: &Hex,
        map: &mut Map,
        window: &gtk::ApplicationWindow,
        area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action) {
        let key = event.get_keyval();
        match key {
            gdk::enums::key::q | gdk::enums::key::Q => {
                (self, Inhibit(false), Action::Quit)
            }
            gdk::enums::key::e | gdk::enums::key::E => {
                if let Some(addr) = self.active_hex {
                    let state = Box::new(ReplaceTile::with_any(map, addr));
                    (state, Inhibit(false), Action::Redraw)
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::u | gdk::enums::key::U => {
                if let Some(addr) = self.active_hex {
                    if let Some(tile) = map.tile_at(addr) {
                        let candidates: Vec<usize> = map
                            .tiles()
                            .iter()
                            .enumerate()
                            .filter(|(_ix, t)| tile.can_upgrade_to(t))
                            .map(|(ix, _t)| ix)
                            .collect();
                        let state = Box::new(ReplaceTile::with_candidates(
                            addr, candidates,
                        ));
                        (state, Inhibit(false), Action::Redraw)
                    } else {
                        (self, Inhibit(false), Action::None)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::s | gdk::enums::key::S => {
                // NOTE: FileChooserNative requires gtk 3.20.
                // let dialog = gtk::FileChooserNative::new(
                //     Some("Save Screenshot"),
                //     Some(window),
                //     gtk::FileChooserAction::Save,
                //     None,
                //     None,
                // );
                let dialog = gtk::FileChooserDialog::with_buttons(
                    Some("Save Screenshot"),
                    Some(window),
                    gtk::FileChooserAction::Save,
                    &[
                        ("_Cancel", gtk::ResponseType::Cancel),
                        ("_Save", gtk::ResponseType::Accept),
                    ],
                );
                let filter_png = gtk::FileFilter::new();
                filter_png.set_name(Some("PNG images"));
                filter_png.add_mime_type("image/png");
                dialog.add_filter(&filter_png);
                let filter_all = gtk::FileFilter::new();
                filter_all.set_name(Some("All files"));
                filter_all.add_pattern("*");
                dialog.add_filter(&filter_all);
                // Suggest a filename that contains the current date and time.
                let now = chrono::Local::now();
                let default_dest =
                    now.format("screenshot-%Y-%m-%d-%H%M%S.png").to_string();
                dialog.set_current_name(default_dest);
                let response = dialog.run();
                let dest_file = if response == gtk::ResponseType::Accept {
                    dialog.get_filename().expect("Couldn't get filename")
                } else {
                    dialog.close();
                    dialog.destroy();
                    return (self, Inhibit(false), Action::None);
                };
                let dest_str = dest_file.to_string_lossy().into_owned();
                dialog.destroy();
                println!("Chose {}", dest_str);
                // Use the same dimensions as the current drawing area.
                let width = area.get_allocated_width();
                let height = area.get_allocated_height();
                let surface =
                    ImageSurface::create(Format::ARgb32, width, height)
                        .expect("Can't create surface");
                let icx = Context::new(&surface);
                // Fill the image with a white background.
                icx.set_source_rgb(1.0, 1.0, 1.0);
                icx.paint();
                // Then draw the current map content.
                self.draw(hex, map, width, height, &icx);
                let mut file = std::fs::File::create(dest_file)
                    .expect(&format!("Couldn't create '{}'", dest_str));
                match surface.write_to_png(&mut file) {
                    Ok(_) => println!("Saved to {}", dest_str),
                    Err(_) => println!("Error saving {}", dest_str),
                }
                (self, Inhibit(false), Action::None)
            }
            gdk::enums::key::Left => {
                // TODO: these are boilerplate, define a common function?
                if let Some(addr) = self.active_hex {
                    let new_addr = map.prev_col(addr);
                    if new_addr == addr {
                        (self, Inhibit(false), Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (self, Inhibit(false), Action::Redraw)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::Right => {
                if let Some(addr) = self.active_hex {
                    let new_addr = map.next_col(addr);
                    if new_addr == addr {
                        (self, Inhibit(false), Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (self, Inhibit(false), Action::Redraw)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::Up => {
                if let Some(addr) = self.active_hex {
                    let new_addr = map.prev_row(addr);
                    if new_addr == addr {
                        (self, Inhibit(false), Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (self, Inhibit(false), Action::Redraw)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::Down => {
                if let Some(addr) = self.active_hex {
                    let new_addr = map.next_row(addr);
                    if new_addr == addr {
                        (self, Inhibit(false), Action::None)
                    } else {
                        self.active_hex = Some(new_addr);
                        (self, Inhibit(false), Action::Redraw)
                    }
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::less | gdk::enums::key::comma => {
                if let Some(addr) = self.active_hex {
                    map.get_mut(addr).map(|hs| hs.rotate_anti_cw());
                    (self, Inhibit(false), Action::Redraw)
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            gdk::enums::key::greater | gdk::enums::key::period => {
                if let Some(addr) = self.active_hex {
                    map.get_mut(addr).map(|hs| hs.rotate_cw());
                    (self, Inhibit(false), Action::Redraw)
                } else {
                    (self, Inhibit(false), Action::None)
                }
            }
            _ => (self, Inhibit(false), Action::None),
        }
    }

    fn button_press(
        self: Box<Self>,
        _hex: &Hex,
        _map: &mut Map,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        _event: &gdk::EventButton,
    ) -> (Box<dyn State>, Inhibit, Action) {
        (self, Inhibit(false), Action::None)
    }
}

impl State for ReplaceTile {
    fn draw(
        &self,
        hex: &Hex,
        map: &Map,
        _width: i32,
        _height: i32,
        ctx: &Context,
    ) {
        for (addr, tile_opt) in map.hex_iter(hex, ctx) {
            if addr == self.active_hex && !self.show_original {
                // Draw the currently-selected replacement tile.
                // NOTE: must account for the current tile's rotation.
                let extra_angle = if let Some(hs) = map.get(addr) {
                    -hs.radians()
                } else {
                    0.0
                };
                ctx.rotate(self.rotation.radians() + extra_angle);
                let tile_ix = self.candidates[self.selected];
                let tile = &map.tiles()[tile_ix];
                tile.draw(ctx, hex);
                if let Some((_tile, tokens)) = tile_opt {
                    // Draw any tokens that have been placed.
                    for (tok, map_token) in tokens.iter() {
                        tile.define_tok_path(&tok, &hex, ctx);
                        map_token.draw_token(&hex, ctx);
                    }
                }
                ctx.rotate(-self.rotation.radians() - extra_angle);
            } else if let Some((tile, tokens)) = tile_opt {
                // Draw the tile and any tokens.
                tile.draw(ctx, hex);
                for (tok, map_token) in tokens.iter() {
                    tile.define_tok_path(&tok, &hex, ctx);
                    map_token.draw_token(&hex, ctx);
                }
            } else {
                // Draw an empty hex.
                // TODO: draw "crosshairs" at hex intersections?
                ctx.set_source_rgb(0.7, 0.7, 0.7);
                hex.define_boundary(ctx);
                ctx.set_line_width(hex.max_d * 0.01);
                ctx.stroke();
            }
        }

        for (addr, _tile_opt) in map.hex_iter(hex, ctx) {
            if self.active_hex == addr {
                // Draw the active hex with a blue border.
                ctx.set_source_rgb(0.0, 0.0, 0.7);
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
    }

    fn key_press(
        mut self: Box<Self>,
        _hex: &Hex,
        map: &mut Map,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        event: &gdk::EventKey,
    ) -> (Box<dyn State>, Inhibit, Action) {
        let key = event.get_keyval();
        match key {
            gdk::enums::key::q | gdk::enums::key::Q => {
                (self, Inhibit(false), Action::Quit)
            }
            gdk::enums::key::Escape => (
                Box::new(Default {
                    active_hex: Some(self.active_hex),
                }),
                Inhibit(false),
                Action::Redraw,
            ),
            gdk::enums::key::Return => {
                if self.show_original {
                    (self, Inhibit(false), Action::None)
                } else {
                    // Replace the original tile with the current selection.
                    let tile_ix = self.candidates[self.selected];
                    let tile_name = map.tiles()[tile_ix].name.clone();
                    map.place_tile(
                        self.active_hex,
                        &tile_name,
                        self.rotation,
                    );
                    (
                        Box::new(Default {
                            active_hex: Some(self.active_hex),
                        }),
                        Inhibit(false),
                        Action::Redraw,
                    )
                }
            }
            gdk::enums::key::o | gdk::enums::key::O => {
                self.show_original = !self.show_original;
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::enums::key::Up => {
                if self.show_original {
                    (self, Inhibit(false), Action::None)
                } else {
                    if self.selected == 0 {
                        self.selected = self.candidates.len() - 1
                    } else {
                        self.selected -= 1
                    }
                    (self, Inhibit(false), Action::Redraw)
                }
            }
            gdk::enums::key::Down => {
                if self.show_original {
                    (self, Inhibit(false), Action::None)
                } else {
                    self.selected += 1;
                    if self.selected >= self.candidates.len() {
                        self.selected = 0;
                    }
                    (self, Inhibit(false), Action::Redraw)
                }
            }
            gdk::enums::key::less | gdk::enums::key::comma => {
                self.rotation = self.rotation.rotate_anti_cw();
                println!("{:?}", self.rotation);
                (self, Inhibit(false), Action::Redraw)
            }
            gdk::enums::key::greater | gdk::enums::key::period => {
                self.rotation = self.rotation.rotate_cw();
                println!("{:?}", self.rotation);
                (self, Inhibit(false), Action::Redraw)
            }
            _ => (self, Inhibit(false), Action::None),
        }
    }

    fn button_press(
        self: Box<Self>,
        _hex: &Hex,
        _map: &mut Map,
        _window: &gtk::ApplicationWindow,
        _area: &gtk::DrawingArea,
        _event: &gdk::EventButton,
    ) -> (Box<dyn State>, Inhibit, Action) {
        (self, Inhibit(false), Action::None)
    }
}
