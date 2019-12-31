use std::cell::RefCell;
use std::collections::HashMap;
use std::env::args;
use std::f64::consts::PI;
use std::rc::Rc;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::DrawingArea;

use cairo::{Context, Format, ImageSurface};

use rusty_train::prelude::*;

fn draw_hexes(state: &State, w: i32, h: i32, cr: &Context) {
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.rectangle(0.0, 0.0, w as f64, h as f64);
    cr.fill();

    let sparse_grid = false;

    let hex = &state.hex;
    let hex_min_d = (3.0 as f64).sqrt() * hex.max_d / 2.0;
    // let flat_top = false;
    let flat_top = true;
    let x0 = if flat_top {
        0.5 * hex.max_d + 10.0
    } else {
        0.5 * hex_min_d + 10.0
    };
    let y0 = if flat_top {
        0.5 * hex_min_d + 10.0
    } else {
        0.5 * hex.max_d + 10.0
    };
    let angle = if flat_top { 0.0 } else { PI / 6.0 };

    for r in 0..6 {
        if sparse_grid && r % 2 == 1 {
            continue;
        }
        for c in 0..14 {
            if sparse_grid && c % 2 == 1 {
                continue;
            }
            let m = cr.get_matrix();

            if flat_top {
                let x = x0 + (c as f64) * hex.max_d * 0.75;
                let y = if c % 2 == 1 {
                    y0 + (r as f64 + 0.5) * hex_min_d
                } else {
                    y0 + (r as f64) * hex_min_d
                };
                cr.translate(x, y);
            } else {
                let x = if r % 2 == 1 {
                    x0 + (c as f64 + 0.5) * hex_min_d
                } else {
                    x0 + (c as f64) * hex_min_d
                };
                let y = y0 + (r as f64) * hex.max_d * 0.75;
                cr.translate(x, y);
            }

            let (tile_ix, tile_angle) = if let UiMode::EditTile {
                ref hex,
                ref candidates,
                ref selected,
                ref angle,
            } = state.ui_mode
            {
                if hex.0 == r && hex.1 == c {
                    (candidates[*selected], *angle)
                } else {
                    let mt = state.map.tiles.get(&(r, c)).unwrap();
                    (mt.tile_ix, mt.angle)
                }
            } else {
                let mt = state.map.tiles.get(&(r, c)).unwrap();
                (mt.tile_ix, mt.angle)
            };
            cr.rotate(angle + tile_angle);

            // Draw the next hex.
            let t = &state.map.catalogue[tile_ix];
            t.draw(cr, &hex);

            let mt = state.map.tiles.get(&(r, c)).unwrap();
            for (token, map_token) in &mt.tokens {
                // Draw a token-specific background.
                t.define_tok_path(&token, hex, cr);
                map_token.draw_token(hex, cr);
            }

            cr.set_matrix(m);
        }
    }

    for r in 0..6 {
        if sparse_grid && r % 2 == 1 {
            continue;
        }
        for c in 0..14 {
            if sparse_grid && c % 2 == 1 {
                continue;
            }
            let m = cr.get_matrix();

            let active = if let UiMode::Normal { active_hex } = state.ui_mode
            {
                active_hex.0 == r && active_hex.1 == c
            } else if let UiMode::EditTile { hex, .. } = state.ui_mode {
                hex.0 == r && hex.1 == c
            } else if let UiMode::EditTokens { hex, .. } = state.ui_mode {
                hex.0 == r && hex.1 == c
            } else {
                false
            };

            if flat_top {
                let x = x0 + (c as f64) * hex.max_d * 0.75;
                let y = if c % 2 == 1 {
                    y0 + (r as f64 + 0.5) * hex_min_d
                } else {
                    y0 + (r as f64) * hex_min_d
                };
                cr.translate(x, y);
            } else {
                let x = if r % 2 == 1 {
                    x0 + (c as f64 + 0.5) * hex_min_d
                } else {
                    x0 + (c as f64) * hex_min_d
                };
                let y = y0 + (r as f64) * hex.max_d * 0.75;
                cr.translate(x, y);
            }

            cr.rotate(angle);

            // Draw a border around the active tile.
            if active {
                if let UiMode::Normal { .. } = state.ui_mode {
                    // Show the active selection with a red border.
                    cr.set_source_rgb(0.7, 0.0, 0.0);
                } else if let UiMode::EditTile { .. } = state.ui_mode {
                    // Show the edit selection with a blue border.
                    cr.set_source_rgb(0.0, 0.0, 0.7);
                } else if let UiMode::EditTokens { .. } = state.ui_mode {
                    // Show the edit selection with a grey border.
                    cr.set_source_rgb(0.3, 0.3, 0.3);
                }
                cr.set_line_width(hex.max_d * 0.01);
                cr.set_line_width(hex.max_d * 0.02);
                hex.define_boundary(cr);
                cr.stroke();

                if let UiMode::EditTokens {
                    ref tokens,
                    selected,
                    ..
                } = state.ui_mode
                {
                    // Retrieve the active tile and orientate it correctly.
                    let mt = state.map.tiles.get(&(r, c)).unwrap();
                    cr.rotate(angle + mt.angle);
                    let t = &state.map.catalogue[mt.tile_ix];

                    // Highlight the active token space.
                    let tok = &tokens[selected];
                    t.define_tok_path(tok, hex, cr);
                    cr.set_source_rgb(0.8, 0.2, 0.2);
                    cr.set_line_width(hex.max_d * 0.025);
                    cr.stroke_preserve();
                }
            } else {
                // Cover all other tiles with a partially-transparent layer.
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.25);
                hex.define_boundary(cr);
                cr.fill();
            }

            cr.set_matrix(m);
        }
    }
}

fn build_ui(application: &gtk::Application) {
    let surface = ImageSurface::create(Format::ARgb32, 600, 600)
        .expect("Can't create surface");
    let icx = Context::new(&surface);
    let state = Rc::new(RefCell::new(State::new(&icx)));

    drawable(application, state.clone(), 1366, 740, move |area, cr| {
        let w = area.get_allocated_width();
        let h = area.get_allocated_height();
        draw_hexes(&state.borrow(), w, h, cr);

        Inhibit(false)
    });
}

fn main() {
    let application =
        gtk::Application::new(Some("rusty_train.main"), Default::default())
            .expect("Initialisation failed...");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}

pub type AppState = Rc<RefCell<State>>;

pub type MapCoord = (usize, usize);
pub type TileId = usize;

#[derive(Debug)]
pub enum UiMode {
    Normal {
        active_hex: MapCoord,
    },
    EditTile {
        hex: MapCoord,
        candidates: Vec<TileId>,
        selected: usize,
        angle: f64,
    },
    EditTokens {
        hex: MapCoord,
        tokens: Vec<rusty_train::tile::Tok>,
        selected: usize,
        old_tokens: HashMap<rusty_train::tile::Tok, MapToken>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapToken {
    LP,
    PO,
    MK,
    N,
}

impl MapToken {
    fn text(&self) -> &str {
        use MapToken::*;

        match self {
            LP => "LP",
            PO => "PO",
            MK => "MK",
            N => "N",
        }
    }

    fn set_bg(&self, ctx: &Context) {
        use MapToken::*;

        match self {
            LP => ctx.set_source_rgb(1.0, 0.5, 0.5),
            PO => ctx.set_source_rgb(0.5, 1.0, 0.5),
            MK => ctx.set_source_rgb(0.5, 1.0, 1.0),
            N => ctx.set_source_rgb(1.0, 0.5, 1.0),
        }
    }

    pub fn draw_token(&self, hex: &Hex, ctx: &Context) {
        let text = self.text();
        self.set_bg(ctx);

        let (x0, y0, x1, y1) = ctx.fill_extents();
        let x = 0.5 * (x0 + x1);
        let y = 0.5 * (y0 + y1);
        ctx.fill_preserve();

        // Draw background elements.
        let stroke_path = ctx.copy_path();
        ctx.save();
        ctx.clip_preserve();
        let radius = hex.max_d * 0.125;
        ctx.set_source_rgb(0.25, 0.6, 0.6);
        ctx.new_path();
        ctx.arc(x - 1.5 * radius, y, 1.0 * radius, 0.0, 2.0 * PI);
        ctx.arc(x + 1.5 * radius, y, 1.0 * radius, 0.0, 2.0 * PI);
        ctx.fill();
        ctx.restore();

        // Redraw the outer black circle.
        ctx.new_path();
        ctx.append_path(&stroke_path);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.set_line_width(hex.max_d * 0.01);
        ctx.stroke_preserve();

        // Draw the token label.
        ctx.select_font_face(
            "Serif",
            cairo::FontSlant::Normal,
            cairo::FontWeight::Bold,
        );
        ctx.set_font_size(10.0);
        let exts = ctx.text_extents(text);
        let x = x - 0.5 * exts.width;
        let y = y + 0.5 * exts.height;
        ctx.move_to(x, y);
        ctx.set_source_rgb(0.0, 0.0, 0.0);
        ctx.show_text(text);
    }

    pub fn next(&self) -> Self {
        use MapToken::*;

        match self {
            LP => PO,
            PO => MK,
            MK => N,
            N => LP,
        }
    }

    pub fn prev(&self) -> Self {
        use MapToken::*;

        match self {
            LP => N,
            PO => LP,
            MK => PO,
            N => MK,
        }
    }

    pub fn first() -> Self {
        MapToken::LP
    }

    pub fn last() -> Self {
        MapToken::N
    }
}

pub struct MapTile {
    tile_ix: usize,
    // TODO: better representation of angle!
    angle: f64,
    tokens: HashMap<rusty_train::tile::Tok, MapToken>,
}

impl MapTile {
    pub fn new(tile_ix: usize, angle: f64) -> Self {
        Self {
            tile_ix,
            angle,
            tokens: HashMap::new(),
        }
    }
}

pub struct Map {
    catalogue: Vec<Tile>,
    tiles: HashMap<MapCoord, MapTile>,
}

pub struct State {
    map: Map,
    hex: Hex,
    ui_mode: UiMode,
}

impl State {
    pub fn new(ctx: &Context) -> Self {
        let hex_max_d = 125.0;
        let hex = Hex::new(hex_max_d);

        let test_tiles = rusty_train::de::test_tiles();
        let test = test_tiles.catalogue(&hex, ctx);
        let tiles_file = "tiles.json";

        // Serialise the new tiles.
        println!("Writing {} ...", tiles_file);
        rusty_train::de::write_pretty(tiles_file, &test_tiles)
            .expect(&format!("Could not write {}", tiles_file));

        // Deserialise the new tiles and check that they are unchanged.
        println!("Reading {} ...", tiles_file);
        let read_tiles = rusty_train::de::read(tiles_file)
            .expect(&format!("Could not read {}", tiles_file));
        let read = read_tiles.catalogue(&hex, ctx);
        println!("{} == test_tiles() : {}", tiles_file, test == read);

        let catalogue = tile_catalogue(&hex, ctx);

        // Compare the new tiles to the original catalogue.
        for (ix, (test, orig)) in
            test.iter().zip(catalogue.iter()).enumerate()
        {
            println!("tile #{} == original : {}", ix, test == orig);
        }

        // let map = ... ?
        let num_rows = 6;
        let num_cols = 14;
        let mut coords = vec![];
        for row in 0..num_rows {
            for col in 0..num_cols {
                coords.push((row as usize, col as usize))
            }
        }
        let map_tiles = coords.into_iter().zip(
            catalogue
                .iter()
                .enumerate()
                // Represent each tile as an index into the catalogue, and
                // set the initial rotation to zero.
                .map(|(ix, _tile)| MapTile::new(ix, 0.0))
                .cycle(),
        );
        let tiles: HashMap<MapCoord, MapTile> = map_tiles.collect();
        let map = Map { catalogue, tiles };
        let ui_mode = UiMode::Normal { active_hex: (0, 0) };
        Self { hex, map, ui_mode }
    }
}

// pub fn connect_key_press<W: IsA<gtk::Widget>>(area: W, state: &AppState) {
//     // Inhibit(false)
// }

pub fn drawable<F>(
    application: &gtk::Application,
    state: AppState,
    width: i32,
    height: i32,
    draw_fn: F,
) where
    F: Fn(&DrawingArea, &Context) -> Inhibit + 'static,
{
    let window = gtk::ApplicationWindow::new(application);
    let drawing_area = Box::new(DrawingArea::new)();

    // TODO: create a HexGrid struct, and have it implement draw_fn,
    // so it can reconcile margins and hex orientation ...

    drawing_area.connect_draw(draw_fn);
    drawing_area.connect_button_press_event(|_widget, event| {
        // TODO: turn coordinates into a hex location (column, row)
        // https://www.redblobgames.com/grids/hexagons/#pixel-to-hex
        let (x, y) = event.get_position();
        // NOTE: 1 = left, 2 = middle, 3 = right.
        let btn = event.get_button();
        println!("button {} at ({:.1}, {:.1})", btn, x, y);
        Inhibit(false)
    });
    window.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
    drawing_area.add_events(gdk::EventMask::BUTTON_PRESS_MASK);

    // drawing_area.connect_key_press_event(|_widget, event| {
    //     let key = event.get_keyval();
    //     println!("key {:?}", key);
    //     Inhibit(false)
    // });
    // NOTE: this event doesn't seem to propagate to the drawing area.
    //
    // From the GTK docs:
    //
    // Propagation differs between event types: key events (GDK_KEY_PRESS,
    // GDK_KEY_RELEASE) are delivered to the top-level GtkWindow; other events
    // are propagated down and up the widget hierarchy in three phases (see
    // GtkPropagationPhase).
    //
    // For key events, the top-level window's default "key-press-event" and
    // "key-release-event" signal handlers handle mnemonics and accelerators
    // first. Other key presses are then passed to
    // gtk_window_propagate_key_event() which propagates the event upwards
    // from the window's current focus widget (gtk_window_get_focus()) to the
    // top-level.
    //
    // https://developer.gnome.org/gtk3/stable/chap-input-handling.html
    //
    let key_state = state.clone();
    let da = drawing_area.clone();
    // TODO: can we make this a method on state instead?!?
    // See how "boxcar willie" does it with connect_click_start ...
    let w = window.clone();
    window.connect_key_press_event(move |_widget, event| {
        let key = event.get_keyval();
        // let shift = event.get_state().contains(gdk::ModifierType::SHIFT_MASK);
        let mut s = key_state.borrow_mut();

        // TODO: this is an issue only inside a closure
        // https://stackoverflow.com/a/36379279
        // https://github.com/rust-lang/rfcs/pull/2229
        // let ui_mode = &mut s.ui_mode;
        // let map = &mut s.map;

        if key == gdk::enums::key::q || key == gdk::enums::key::Q {
            w.destroy();
            return Inhibit(false);
        }

        match s.ui_mode {
            // TODO: map keys to actions, so that we can direct <shift>+left
            // to rotate.
            UiMode::Normal { ref mut active_hex } => match key {
                gdk::enums::key::Left => {
                    if active_hex.1 > 0 {
                        active_hex.1 -= 1;
                        da.queue_draw();
                    }
                }
                gdk::enums::key::Right => {
                    if active_hex.1 < 13 {
                        active_hex.1 += 1;
                        da.queue_draw();
                    }
                }
                gdk::enums::key::Up => {
                    if active_hex.0 > 0 {
                        active_hex.0 -= 1;
                        da.queue_draw();
                    }
                }
                gdk::enums::key::Down => {
                    if active_hex.0 < 5 {
                        active_hex.0 += 1;
                        da.queue_draw();
                    }
                }
                gdk::enums::key::less | gdk::enums::key::comma => {
                    let active_hex = &active_hex.clone();
                    s.map
                        .tiles
                        .get_mut(active_hex)
                        .map(|mt| mt.angle -= PI / 3.0);
                    da.queue_draw();
                }
                gdk::enums::key::greater | gdk::enums::key::period => {
                    let active_hex = &active_hex.clone();
                    s.map
                        .tiles
                        .get_mut(active_hex)
                        .map(|mt| mt.angle += PI / 3.0);
                    da.queue_draw();
                }
                gdk::enums::key::e | gdk::enums::key::E => {
                    // NOTE: switch to edit mode!
                    // TODO: smart selection of candidates --- next_phase()?
                    // If no candidates, automatically exit.

                    // pub struct Map {
                    //     catalogue: Vec<Tile>,
                    //     // TODO: better representation of angle!
                    //     tiles: HashMap<MapCoord, (usize, f64)>,
                    // }

                    let hex = active_hex.clone();
                    let ix = s.map.tiles.get(&hex).unwrap().tile_ix;
                    let tile = &s.map.catalogue[ix];
                    match tile.colour.next_phase() {
                        Some(colour) => {
                            let candidates = s
                                .map
                                .catalogue
                                .iter()
                                .enumerate()
                                .filter(|(_ix, tile)| tile.colour == colour)
                                .map(|(ix, _tile)| ix)
                                .collect();
                            s.ui_mode = UiMode::EditTile {
                                hex: hex,
                                // candidates: (0..s.map.catalogue.len()).collect(),
                                candidates: candidates,
                                selected: 0,
                                angle: 0.0,
                            };
                            da.queue_draw();
                        }
                        _ => {}
                    }
                }
                gdk::enums::key::t | gdk::enums::key::T => {
                    // NOTE: switch to token mode!
                    let hex = active_hex.clone();
                    let mt = s.map.tiles.get(&hex).unwrap();
                    let ix = mt.tile_ix;
                    let tile = &s.map.catalogue[ix];
                    let tokens = tile.toks();
                    if tokens.len() > 0 {
                        s.ui_mode = UiMode::EditTokens {
                            hex,
                            tokens,
                            selected: 0,
                            old_tokens: mt.tokens.clone(),
                        };
                        da.queue_draw();
                    }
                }
                gdk::enums::key::s | gdk::enums::key::S => {
                    // Take a screenshot.
                    // NOTE: may want to reserve 'S' for saving?
                    let surface =
                        ImageSurface::create(Format::ARgb32, width, height)
                            .expect("Can't create surface");
                    let icx = Context::new(&surface);
                    let dest_file = "screenshot.png";
                    draw_hexes(&s, width, height, &icx);
                    let mut file = std::fs::File::create(dest_file)
                        .expect("Couldn't create 'hexes.png'");
                    match surface.write_to_png(&mut file) {
                        Ok(_) => println!("Saved to {}", dest_file),
                        Err(_) => println!("Error saving {}", dest_file),
                    }
                }
                _ => {}
            },
            UiMode::EditTile {
                ref hex,
                ref candidates,
                ref mut selected,
                ref mut angle,
            } => match key {
                gdk::enums::key::Escape => {
                    // NOTE: cancel edit mode!
                    s.ui_mode = UiMode::Normal { active_hex: *hex };
                    da.queue_draw();
                }
                gdk::enums::key::Return => {
                    // NOTE: apply changes and exit from edit mode!
                    let hex = hex.clone();
                    let tile_ix = candidates[*selected];
                    let angle = *angle;
                    s.map.tiles.insert(hex, MapTile::new(tile_ix, angle));
                    s.ui_mode = UiMode::Normal { active_hex: hex };
                    da.queue_draw();
                }
                gdk::enums::key::Up => {
                    if *selected == 0 {
                        *selected = candidates.len() - 1
                    } else {
                        *selected -= 1
                    }
                    da.queue_draw();
                }
                gdk::enums::key::Down => {
                    *selected += 1;
                    if *selected >= candidates.len() {
                        *selected = 0
                    }
                    da.queue_draw();
                }
                gdk::enums::key::less | gdk::enums::key::comma => {
                    *angle -= PI / 3.0;
                    da.queue_draw();
                }
                gdk::enums::key::greater | gdk::enums::key::period => {
                    *angle += PI / 3.0;
                    da.queue_draw();
                }
                _ => {}
            },
            UiMode::EditTokens {
                ref hex,
                ref tokens,
                ref mut selected,
                ref mut old_tokens,
            } => match key {
                gdk::enums::key::Escape => {
                    // NOTE: revert the edits and exit this mode.
                    let hex = hex.clone();
                    let repl = old_tokens.drain().collect();
                    s.map.tiles.get_mut(&hex).unwrap().tokens = repl;
                    s.ui_mode = UiMode::Normal { active_hex: hex };
                    da.queue_draw();
                }
                gdk::enums::key::Return => {
                    // NOTE: no changes to apply, just exit this mode.
                    s.ui_mode = UiMode::Normal { active_hex: *hex };
                    da.queue_draw();
                }
                gdk::enums::key::Left => {
                    if *selected == 0 {
                        *selected = tokens.len() - 1
                    } else {
                        *selected -= 1
                    }
                    da.queue_draw();
                }
                gdk::enums::key::Right => {
                    *selected += 1;
                    if *selected >= tokens.len() {
                        *selected = 0
                    }
                    da.queue_draw();
                }
                gdk::enums::key::Up => {
                    let tok = tokens[*selected].clone();
                    let hex = hex.clone();
                    s.map.tiles.get_mut(&hex).map(|mt| {
                        let next = mt
                            .tokens
                            .get(&tok)
                            .map(|t| t.next())
                            .unwrap_or(MapToken::first());
                        mt.tokens.insert(tok, next);
                        da.queue_draw();
                    });
                }
                gdk::enums::key::Down => {
                    let tok = tokens[*selected].clone();
                    let hex = hex.clone();
                    s.map.tiles.get_mut(&hex).map(|mt| {
                        let next = mt
                            .tokens
                            .get(&tok)
                            .map(|t| t.prev())
                            .unwrap_or(MapToken::last());
                        mt.tokens.insert(tok, next);
                        da.queue_draw();
                    });
                }
                gdk::enums::key::_0
                | gdk::enums::key::KP_0
                | gdk::enums::key::BackSpace => {
                    let tok = tokens[*selected].clone();
                    let hex = hex.clone();
                    s.map.tiles.get_mut(&hex).map(|mt| {
                        mt.tokens.remove(&tok);
                        da.queue_draw();
                    });
                }
                _ => {}
            },
        }
        Inhibit(false)
    });
    // window.connect_key_press_event(move |_widget, event| {
    //     let key = event.get_keyval();
    //     let shift = event.get_state().contains(gdk::ModifierType::SHIFT_MASK);
    //     match key {
    //         gdk::enums::key::Left => {
    //             println!("Left");
    //             let mut s = key_state.borrow_mut();
    //             if shift {
    //                 // NOTE: rotate if shift is being held down ...
    //                 let key = (s.active_row, s.active_col);
    //                 let angle = s.angle.entry(key).or_insert(0.0);
    //                 // TODO: better representation, use an enum
    //                 *angle -= PI / 3.0;
    //                 da.queue_draw();
    //             } else {
    //                 if s.active_col > 0 {
    //                     s.active_col -= 1;
    //                     da.queue_draw();
    //                 }
    //             }
    //         }
    //         gdk::enums::key::Right => {
    //             println!("Right");
    //             let mut s = key_state.borrow_mut();
    //             if shift {
    //                 // NOTE: rotate if shift is being held down ...
    //                 let key = (s.active_row, s.active_col);
    //                 let angle = s.angle.entry(key).or_insert(0.0);
    //                 // TODO: better representation, use an enum
    //                 *angle += PI / 3.0;
    //                 da.queue_draw();
    //             } else {
    //                 if s.active_col < 13 {
    //                     s.active_col += 1;
    //                     da.queue_draw();
    //                 }
    //             }
    //         }
    //         gdk::enums::key::Up => {
    //             println!("Up");
    //             let mut s = key_state.borrow_mut();
    //             if s.active_row > 0 {
    //                 s.active_row -= 1;
    //                 da.queue_draw();
    //             }
    //         }
    //         gdk::enums::key::Down => {
    //             println!("Down");
    //             let mut s = key_state.borrow_mut();
    //             if s.active_row < 5 {
    //                 s.active_row += 1;
    //                 da.queue_draw();
    //             }
    //         }
    //         gdk::enums::key::less | gdk::enums::key::comma => {
    //             println!("Less");
    //             let mut s = key_state.borrow_mut();
    //             let key = (s.active_row, s.active_col);
    //             let angle = s.angle.entry(key).or_insert(0.0);
    //             // TODO: better representation, use an enum
    //             *angle -= PI / 3.0;
    //             da.queue_draw();
    //         }
    //         gdk::enums::key::greater | gdk::enums::key::period => {
    //             println!("Greater");
    //             let mut s = key_state.borrow_mut();
    //             let key = (s.active_row, s.active_col);
    //             let angle = s.angle.entry(key).or_insert(0.0);
    //             // TODO: better representation, use an enum
    //             *angle += PI / 3.0;
    //             da.queue_draw();
    //         }
    //         // TODO: cycle through tiles?
    //         gdk::enums::key::Page_Down => {}
    //         gdk::enums::key::Page_Up => {}
    //         _ => println!("key {:?}", key),
    //     }
    //     Inhibit(false)
    // });
    window.add_events(gdk::EventMask::KEY_PRESS_MASK);
    // drawing_area.add_events(gdk::EventMask::KEY_PRESS_MASK);

    window.set_default_size(width, height);

    window.add(&drawing_area);
    window.show_all();
}
