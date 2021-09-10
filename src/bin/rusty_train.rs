use std::io::Write;

use gtk::prelude::*;
use gtk::DrawingArea;

use navig18xx::prelude::*;

pub fn build_ui(application: &gtk::Application) {
    // NOTE: make this more like the doc example in n18ui.

    let games: Vec<Box<dyn Game>> = vec![
        Box::new(navig18xx::game::new_1861()),
        Box::new(navig18xx::game::new_1867()),
    ];

    // NOTE: instead of using Rc<RefCell<UI>> to share a mutable UI value, use
    // channels to send messages to a single event-handler that owns and
    // mutates the UI state.
    let hex_width = 125.0;
    let hex = Hex::new(hex_width);
    let state = UI::new(hex, games);
    run(application, state);
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

    let application =
        gtk::Application::new(Some("rusty_train.bin"), Default::default());

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run();
}

pub enum UiEvent {
    ButtonPress(navig18xx::ui::ButtonPress),
    KeyPress(navig18xx::ui::KeyPress),
    PingCurrentState(navig18xx::ui::PingDest),
}

pub fn run(application: &gtk::Application, mut state: UI) {
    let window = gtk::ApplicationWindow::new(application);
    let bar = gtk::HeaderBar::new();
    let adj: Option<&gtk::Adjustment> = None;
    let scrolled_win = gtk::ScrolledWindow::new(adj, adj);
    let drawing_area = Box::new(DrawingArea::new)();

    let (width, height) = state.map_size();

    bar.set_title(Some("Rusty Train"));
    bar.set_decoration_layout(Some("menu:close"));
    bar.set_show_close_button(true);
    window.set_titlebar(Some(&bar));

    // Set the minimum size of the drawing area to the required canvas size.
    drawing_area.set_size_request(width, height);

    // Create a channel to pass UI events to the `state` event handlers.
    let (tx, rx) =
        glib::MainContext::sync_channel(glib::PRIORITY_DEFAULT, 100);

    // Let the UI draw on the window.
    let surface = state.surface();
    drawing_area.connect_draw(move |_da, ctx| {
        let surf = surface.read().expect("Could not access drawing surface");
        ctx.set_source_surface(&surf, 0.0, 0.0).unwrap();
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

    let area_ = drawing_area.clone();
    let win_ = window.clone();

    // Create a second channel for sending unit values, which can be used to
    // trigger non-UI events, such as messages from tasks in other threads.
    // We provide the sender to each `state` method, so that they can make use
    // of it as necessary.
    let (ping_tx, ping_rx) =
        glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    // Pass each "ping" event to the current UI state.
    ping_rx.attach(None, move |dest| {
        tx.send(UiEvent::PingCurrentState(dest))
            .expect("Could not send Ping event");
        glib::source::Continue(true)
    });

    rx.attach(None, move |event| {
        let action = match event {
            UiEvent::ButtonPress(event) => {
                state.button_press_action(&win_, &area_, &event, &ping_tx)
            }
            UiEvent::KeyPress(event) => {
                state.key_press_action(&win_, &area_, &event, &ping_tx)
            }
            UiEvent::PingCurrentState(dest) => {
                state.ping(dest, &win_, &area_, &ping_tx)
            }
        };
        state.handle_action(&win_, &area_, action);
        glib::source::Continue(true)
    });

    // Display the window.
    window.set_default_size(width, height);
    scrolled_win.add(&drawing_area);
    window.add(&scrolled_win);
    drawing_area.queue_draw();
    window.show_all();
}
