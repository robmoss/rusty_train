use std::cell::RefCell;
use std::env::args;
use std::rc::Rc;

use gio::prelude::*;
use gtk::prelude::*;
use gtk::DrawingArea;

use cairo::Context;
use cairo::{Format, ImageSurface};

use std::io::Write;

use rusty_train::prelude::*;

type App = Rc<RefCell<UI>>;

pub fn build_ui(application: &gtk::Application) {
    // NOTE: make this more like the doc example in rusty_train::ui.

    let hex_width: i32 = 125;
    let hex = Hex::new(hex_width as f64);
    let game = rusty_game::_1867::Game::new(&hex);
    let map = game.create_map(&hex);

    let num_rows = map.max_row;
    let num_cols = map.max_col;
    let sw = ((num_cols as f64) * hex.min_d) as i32;
    let sh = (num_rows as i32) * hex_width;

    let game_box = Box::new(game);

    // NOTE: instead of using Rc<RefCell<UI>> to share a mutable UI value, use
    // channels to send messages to a single event-handler that owns and
    // mutates the UI state.
    let use_refcell = false;
    if !use_refcell {
        let state = UI::new(hex, game_box, map);
        run(application, state, sw, sh);
        return;
    }

    let state = Rc::new(RefCell::new(UI::new(hex, game_box, map)));

    let surface = ImageSurface::create(Format::ARgb32, sw, sh)
        .expect("Can't create surface");
    let icx = Context::new(&surface);

    drawable(application, state.clone(), sw, sh, move |area, cr| {
        let w = area.get_allocated_width();
        let h = area.get_allocated_height();
        if let Ok(ui) = state.try_borrow() {
            ui.draw(cr);
            // Copy what the UI has just drawn to the image surface.
            // TODO: do we need to resize or recreate the surface when the
            // user zooms in or out?
            // It needs to be at least as big as (w, h).
            // Can the move lambda capture surface and icx as mutable
            // variables and create a new surface/context are required?
            icx.set_source_surface(&cr.get_target(), 0.0, 0.0);
            icx.rectangle(0.0, 0.0, w.into(), h.into());
            icx.set_operator(cairo::Operator::Source);
            icx.fill();
        } else {
            // NOTE: this occurs when modal dialog windows are being displayed
            // (e.g., for saving or loading files).
            // Copy what the UI most recently drew from the image surface.
            cr.save();
            cr.set_source_surface(&surface, 0.0, 0.0);
            cr.paint();
            cr.restore();
        }

        Inhibit(false)
    });
}

pub fn main() {
    // Default to logging all messages up to ``log::Level::Info``, using a
    // custom message format.
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .format(|buf, record| {
        writeln!(
            buf,
            "{} [{}] {}",
            chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
            record.level(),
            record.args()
        )
    })
    .init();

    let application = gtk::Application::new(
        Some("rusty_train.test_ui"),
        Default::default(),
    )
    .expect("Initialisation failed...");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}

pub enum UiEvent {
    ButtonPress { event: gdk::EventButton },
    KeyPress { event: gdk::EventKey },
}

pub fn run(
    application: &gtk::Application,
    mut state: UI,
    width: i32,
    height: i32,
) {
    let window = gtk::ApplicationWindow::new(application);
    let bar = gtk::HeaderBar::new();
    let adj: Option<&gtk::Adjustment> = None;
    let scrolled_win = gtk::ScrolledWindow::new(adj, adj);
    let drawing_area = Box::new(DrawingArea::new)();

    let surf = cairo::ImageSurface::create(
        cairo::Format::ARgb32,
        width * 2,
        height * 2,
    )
    .expect("Could not create ImageSurface");
    let state_ctx = cairo::Context::new(&surf);

    state_ctx.set_source_rgb(1.0, 1.0, 1.0);
    state_ctx.rectangle(0.0, 0.0, 2.0 * width as f64, 2.0 * height as f64);
    state_ctx.fill();
    state.draw(&state_ctx);

    bar.set_title(Some("Rusty Train"));
    bar.set_decoration_layout(Some("menu:close"));
    bar.set_show_close_button(true);
    window.set_titlebar(Some(&bar));

    // Set the minimum size of the drawing area to the required canvas size.
    drawing_area.set_size_request(width, height);

    let (tx, rx) =
        glib::MainContext::sync_channel(glib::PRIORITY_DEFAULT, 100);

    // Let the UI draw on the window.
    drawing_area.connect_draw(move |_da, ctx| {
        ctx.set_source_surface(&surf, 0.0, 0.0);
        ctx.paint();
        Inhibit(false)
    });

    // Let the UI handle mouse button events.
    let tx_ = tx.clone();
    drawing_area.connect_button_press_event(move |_widget, event| {
        tx_.send(UiEvent::ButtonPress {
            event: event.clone(),
        })
        .expect("Could not send ButtonPress event");
        Inhibit(false)
    });
    window.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
    scrolled_win.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
    drawing_area.add_events(gdk::EventMask::BUTTON_PRESS_MASK);

    // Let the UI handle keyboard events.
    let tx_ = tx.clone();
    window.connect_key_press_event(move |_widget, event| {
        tx_.send(UiEvent::KeyPress {
            event: event.clone(),
        })
        .expect("Could not send KeyPress event");
        Inhibit(false)
    });
    window.add_events(gdk::EventMask::KEY_PRESS_MASK);

    let area_ = drawing_area.clone();
    let win_ = window.clone();
    rx.attach(None, move |event| {
        let (_inhibit, action) = match event {
            UiEvent::ButtonPress { event } => {
                state.button_press_action(&win_, &area_, &event)
            }
            UiEvent::KeyPress { event } => {
                state.key_press_action(&win_, &area_, &event)
            }
        };
        if action == rusty_ui::Action::Redraw {
            state.draw(&state_ctx);
            area_.queue_draw();
        } else {
            state.handle_action(&win_, &area_, action);
        }
        glib::source::Continue(true)
    });

    // Display the window.
    window.set_default_size(width, height);
    scrolled_win.add(&drawing_area);
    window.add(&scrolled_win);
    drawing_area.queue_draw();
    window.show_all();
}

pub fn drawable<F>(
    application: &gtk::Application,
    state: App,
    width: i32,
    height: i32,
    draw_fn: F,
) where
    F: Fn(&DrawingArea, &Context) -> Inhibit + 'static,
{
    let window = gtk::ApplicationWindow::new(application);
    let bar = gtk::HeaderBar::new();
    let adj: Option<&gtk::Adjustment> = None;
    let scrolled_win = gtk::ScrolledWindow::new(adj, adj);
    let drawing_area = Box::new(DrawingArea::new)();

    bar.set_title(Some("Rusty Train"));
    bar.set_decoration_layout(Some("menu:close"));
    bar.set_show_close_button(true);
    window.set_titlebar(Some(&bar));

    // Set the minimum size of the drawing area to the required canvas size.
    drawing_area.set_size_request(width, height);

    // Let the UI draw on the window.
    drawing_area.connect_draw(draw_fn);

    // Let the UI handle mouse button events.
    let app = state.clone();
    let da = drawing_area.clone();
    let w = window.clone();
    drawing_area.connect_button_press_event(move |_widget, event| {
        let mut ui = app.borrow_mut();
        ui.button_press(&w, &da, event)
    });
    window.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
    scrolled_win.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
    drawing_area.add_events(gdk::EventMask::BUTTON_PRESS_MASK);

    // Let the UI handle keyboard events.
    let app = state.clone();
    let da = drawing_area.clone();
    let w = window.clone();
    window.connect_key_press_event(move |_widget, event| {
        let mut ui = app.borrow_mut();
        ui.key_press(&w, &da, event)
    });
    window.add_events(gdk::EventMask::KEY_PRESS_MASK);

    // Display the window.
    window.set_default_size(width, height);
    scrolled_win.add(&drawing_area);
    window.add(&scrolled_win);
    window.show_all();
}
