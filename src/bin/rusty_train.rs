use std::io::Write;

use gtk::prelude::*;
use gtk::DrawingArea;

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

    let application =
        gtk::Application::new(Some("rusty_train.bin"), Default::default());

    application.connect_activate(|app| {
        build(app);
    });

    application.run();
}

pub enum UiEvent {
    ButtonPress(navig18xx::ui::ButtonPress),
    KeyPress(navig18xx::ui::KeyPress),
    PingCurrentState(navig18xx::ui::PingDest),
}

pub fn build(application: &gtk::Application) {
    let games = navig18xx::game::games();

    let window = gtk::ApplicationWindow::new(application);
    let bar = gtk::HeaderBar::new();
    let adj: Option<&gtk::Adjustment> = None;
    let scrolled_win = gtk::ScrolledWindow::new(adj, adj);
    let drawing_area = Box::new(DrawingArea::new)();
    scrolled_win.set_child(Some(&drawing_area));

    // Create a second channel for sending "pings", which can be used to
    // trigger non-UI events, such as messages from tasks in other threads.
    // We provide the sender to each `state` method, so that they can make use
    // of it as necessary.
    let (ping_tx, ping_rx) =
        glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let win = window.clone().upcast::<gtk::Window>();
    let controller = navig18xx::ui::control::Gtk3Controller::new(
        win,
        drawing_area.clone(),
        ping_tx,
    );
    let mut ui = navig18xx::ui::UserInterface::new(
        games,
        controller,
        Default::default(),
    );
    ui.draw();

    bar.set_title(Some("Rusty Train"));
    bar.set_decoration_layout(Some("menu:close"));
    bar.set_show_close_button(true);
    window.set_titlebar(Some(&bar));

    // Create a channel to pass UI events to the `state` event handlers.
    let (tx, rx) =
        glib::MainContext::sync_channel(glib::PRIORITY_DEFAULT, 100);

    // Let the UI draw on the window.
    let surface = ui.canvas.surface();
    drawing_area.connect_draw(move |_da, ctx| {
        let surf = surface.read().expect("Could not access drawing surface");
        ctx.set_source_surface(&*surf, 0.0, 0.0).unwrap();
        ctx.paint().unwrap();
        Inhibit(false)
    });

    // Let the UI handle mouse button events.
    let tx_ = tx.clone();
    drawing_area.connect_button_press_event(move |_widget, event| {
        tx_.send(UiEvent::ButtonPress(event.into()))
            .expect("Could not send ButtonPress event");
        Inhibit(false)
    });
    window.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
    scrolled_win.add_events(gdk::EventMask::BUTTON_PRESS_MASK);
    drawing_area.add_events(gdk::EventMask::BUTTON_PRESS_MASK);

    // Let the UI handle keyboard events.
    let tx_ = tx.clone();
    window.connect_key_press_event(move |_widget, event| {
        tx_.send(UiEvent::KeyPress(event.into()))
            .expect("Could not send KeyPress event");
        Inhibit(false)
    });
    window.add_events(gdk::EventMask::KEY_PRESS_MASK);

    // Pass each "ping" event to the current UI state.
    ping_rx.attach(None, move |dest| {
        tx.send(UiEvent::PingCurrentState(dest))
            .expect("Could not send Ping event");
        glib::source::Continue(true)
    });

    // Show the starting message, rather than the map content.
    let mut start_visible = true;
    let start_widget = start_message();
    window.set_child(Some(&start_widget));

    // Dispatch events to the appropriate handler.
    // Note that this closure owns `ui_state`.
    let _window = window.clone();
    rx.attach(None, move |event| {
        let response = match event {
            UiEvent::ButtonPress(event) => ui.handle_button_press(&event),
            UiEvent::KeyPress(event) => ui.handle_key_press(&event),
            UiEvent::PingCurrentState(dest) => ui.ping(dest),
        };
        ui.respond(response);

        // When a game is started or loaded, hide the starting message and
        // show the map content instead.
        if start_visible && ui.state.as_start().is_none() {
            _window.remove(&start_widget);
            _window.add(&scrolled_win);
            _window.show_all();
            start_visible = false;
        }

        glib::source::Continue(true)
    });

    // Display the window.
    window.show_all();
}

/// Returns a Grid containing labels that identify the available key bindings
/// when Rusty Train is launched.
fn start_message() -> gtk::Grid {
    let grid = gtk::Grid::builder()
        .expand(true)
        .halign(gtk::Align::Center)
        .valign(gtk::Align::Center)
        .build();
    ["Ctrl+N", "Ctrl+O", "Q"]
        .iter()
        .enumerate()
        .for_each(|(ix, text)| {
            let label = gtk::Label::builder()
                .use_markup(true)
                .selectable(false)
                .label(&format!("<b>{}:</b> ", text))
                .expand(true)
                .halign(gtk::Align::End)
                .build();
            grid.attach(&label, 0, ix as i32, 1, 1);
        });
    ["Start a new game", "Load a saved game", "Quit"]
        .iter()
        .enumerate()
        .for_each(|(ix, text)| {
            let label = gtk::Label::builder()
                .selectable(false)
                .label(text)
                .expand(true)
                .halign(gtk::Align::Start)
                .build();
            grid.attach(&label, 1, ix as i32, 1, 1);
        });
    grid
}
