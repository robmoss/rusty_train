use n18game::Game;
use n18route::Trains;

use crate::PingDest;

mod _gtk3;

#[doc(inline)]
pub use _gtk3::Gtk3Controller;

#[derive(Clone)]
pub enum PingSender {
    Glib(glib::Sender<PingDest>),
    Mpsc(std::sync::mpsc::Sender<PingDest>),
    IgnorePings,
}

impl PingSender {
    pub fn send_ping(
        &self,
        dest: PingDest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use PingSender::*;
        match self {
            Glib(sender) => sender.send(dest),
            Mpsc(sender) => sender.send(dest),
            IgnorePings => Ok(()),
        }
        .map_err(|e| e.into())
    }
}

pub trait UiController {
    fn window_title(&self) -> Option<String>;

    fn set_window_title(&mut self, title: &str);

    fn quit(&mut self);

    fn redraw(&self);

    fn resize(&mut self, width: i32, height: i32);

    fn ping_tx(&self) -> PingSender;

    fn select_string<F>(
        &mut self,
        title: &str,
        strings: &[&str],
        callback: F,
    ) where
        Self: Sized,
        F: Fn(Option<String>) + 'static;

    fn select_index<F>(&mut self, title: &str, strings: &[&str], callback: F)
    where
        Self: Sized,
        F: Fn(Option<usize>) + 'static;

    fn select_trains<F>(&mut self, game: &dyn Game, title: &str, callback: F)
    where
        Self: Sized,
        F: Fn(Option<(Trains, Vec<bool>)>) + 'static;

    fn select_phase<F>(&mut self, game: &dyn Game, callback: F)
    where
        Self: Sized,
        F: Fn(Option<usize>) + 'static;

    fn select_screenshot_save<F>(
        &mut self,
        title: &str,
        default_path: Option<&str>,
        callback: F,
    ) where
        Self: Sized,
        F: Fn(Option<std::path::PathBuf>) + 'static;

    fn select_game_save<F>(
        &mut self,
        title: &str,
        default_path: Option<&str>,
        callback: F,
    ) where
        Self: Sized,
        F: Fn(Option<std::path::PathBuf>) + 'static;

    fn select_game_load<F>(
        &mut self,
        title: &str,
        default_path: Option<&str>,
        callback: F,
    ) where
        Self: Sized,
        F: Fn(Option<std::path::PathBuf>) + 'static;
}

pub enum Controller {
    Gtk3(Gtk3Controller),
    Dummy(DummyController),
}

impl From<Gtk3Controller> for Controller {
    fn from(ctrl: Gtk3Controller) -> Self {
        Controller::Gtk3(ctrl)
    }
}

impl From<DummyController> for Controller {
    fn from(ctrl: DummyController) -> Self {
        Controller::Dummy(ctrl)
    }
}

impl UiController for Controller {
    fn quit(&mut self) {
        use Controller::*;
        match self {
            Gtk3(ctrl) => ctrl.quit(),
            Dummy(ctrl) => ctrl.quit(),
        }
    }

    fn redraw(&self) {
        use Controller::*;
        match self {
            Gtk3(ctrl) => ctrl.redraw(),
            Dummy(ctrl) => ctrl.redraw(),
        }
    }

    fn set_window_title(&mut self, title: &str) {
        use Controller::*;
        match self {
            Gtk3(ctrl) => ctrl.set_window_title(title),
            Dummy(ctrl) => ctrl.set_window_title(title),
        }
    }

    fn window_title(&self) -> Option<String> {
        use Controller::*;
        match self {
            Gtk3(ctrl) => ctrl.window_title(),
            Dummy(ctrl) => ctrl.window_title(),
        }
    }

    fn resize(&mut self, width: i32, height: i32) {
        use Controller::*;
        match self {
            Gtk3(ctrl) => ctrl.resize(width, height),
            Dummy(ctrl) => ctrl.resize(width, height),
        }
    }

    fn ping_tx(&self) -> PingSender {
        use Controller::*;
        match self {
            Gtk3(ctrl) => ctrl.ping_tx(),
            Dummy(ctrl) => ctrl.ping_tx(),
        }
    }

    fn select_string<F>(&mut self, title: &str, strings: &[&str], callback: F)
    where
        F: Fn(Option<String>) + 'static,
    {
        use Controller::*;
        match self {
            Gtk3(ctrl) => ctrl.select_string(title, strings, callback),
            Dummy(ctrl) => ctrl.select_string(title, strings, callback),
        }
    }

    fn select_index<F>(&mut self, title: &str, strings: &[&str], callback: F)
    where
        F: Fn(Option<usize>) + 'static,
    {
        use Controller::*;
        match self {
            Gtk3(ctrl) => ctrl.select_index(title, strings, callback),
            Dummy(ctrl) => ctrl.select_index(title, strings, callback),
        }
    }

    fn select_trains<F>(&mut self, game: &dyn Game, title: &str, callback: F)
    where
        F: Fn(Option<(Trains, Vec<bool>)>) + 'static,
    {
        use Controller::*;
        match self {
            Gtk3(ctrl) => ctrl.select_trains(game, title, callback),
            Dummy(ctrl) => ctrl.select_trains(game, title, callback),
        }
    }

    fn select_phase<F>(&mut self, game: &dyn Game, callback: F)
    where
        F: Fn(Option<usize>) + 'static,
    {
        use Controller::*;
        match self {
            Gtk3(ctrl) => ctrl.select_phase(game, callback),
            Dummy(ctrl) => ctrl.select_phase(game, callback),
        }
    }

    fn select_screenshot_save<F>(
        &mut self,
        title: &str,
        default_path: Option<&str>,
        callback: F,
    ) where
        F: Fn(Option<std::path::PathBuf>) + 'static,
    {
        use Controller::*;
        match self {
            Gtk3(ctrl) => {
                ctrl.select_screenshot_save(title, default_path, callback)
            }
            Dummy(ctrl) => {
                ctrl.select_screenshot_save(title, default_path, callback)
            }
        }
    }

    fn select_game_save<F>(
        &mut self,
        title: &str,
        default_path: Option<&str>,
        callback: F,
    ) where
        F: Fn(Option<std::path::PathBuf>) + 'static,
    {
        use Controller::*;
        match self {
            Gtk3(ctrl) => {
                ctrl.select_game_save(title, default_path, callback)
            }
            Dummy(ctrl) => {
                ctrl.select_game_save(title, default_path, callback)
            }
        }
    }

    fn select_game_load<F>(
        &mut self,
        title: &str,
        default_path: Option<&str>,
        callback: F,
    ) where
        F: Fn(Option<std::path::PathBuf>) + 'static,
    {
        use Controller::*;
        match self {
            Gtk3(ctrl) => {
                ctrl.select_game_load(title, default_path, callback)
            }
            Dummy(ctrl) => {
                ctrl.select_game_load(title, default_path, callback)
            }
        }
    }
}

#[derive(Default)]
pub struct DummyController {
    game_load: Option<std::path::PathBuf>,
    game_save: Option<std::path::PathBuf>,
    screenshot_save: Option<std::path::PathBuf>,
    phase: Option<usize>,
    index: Option<usize>,
    string: Option<String>,
    trains: Option<(Trains, Vec<bool>)>,
}

impl DummyController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_game_load_path(&mut self, path: Option<std::path::PathBuf>) {
        self.game_load = path
    }

    pub fn set_game_save_path(&mut self, path: Option<std::path::PathBuf>) {
        self.game_save = path
    }

    pub fn set_screenshot_save_path(
        &mut self,
        path: Option<std::path::PathBuf>,
    ) {
        self.screenshot_save = path
    }

    pub fn set_phase(&mut self, phase: Option<usize>) {
        self.phase = phase
    }
}

impl UiController for DummyController {
    fn quit(&mut self) {}

    fn redraw(&self) {}

    fn set_window_title(&mut self, _title: &str) {}

    fn window_title(&self) -> Option<String> {
        None
    }

    fn resize(&mut self, _width: i32, _height: i32) {}

    fn ping_tx(&self) -> PingSender {
        PingSender::IgnorePings
    }

    fn select_string<F>(
        &mut self,
        _title: &str,
        _strings: &[&str],
        callback: F,
    ) where
        F: Fn(Option<String>) + 'static,
    {
        callback(self.string.clone())
    }

    fn select_index<F>(
        &mut self,
        _title: &str,
        _strings: &[&str],
        callback: F,
    ) where
        F: Fn(Option<usize>) + 'static,
    {
        callback(self.index)
    }

    fn select_trains<F>(
        &mut self,
        _game: &dyn Game,
        _title: &str,
        callback: F,
    ) where
        Self: Sized,
        F: Fn(Option<(Trains, Vec<bool>)>) + 'static,
    {
        let trains = self
            .trains
            .as_ref()
            .map(|(trains, bonuses)| (trains.clone(), bonuses.clone()));
        callback(trains)
    }

    fn select_phase<F>(&mut self, _game: &dyn Game, callback: F)
    where
        Self: Sized,
        F: Fn(Option<usize>) + 'static,
    {
        callback(self.phase)
    }

    fn select_screenshot_save<F>(
        &mut self,
        _title: &str,
        _default_path: Option<&str>,
        callback: F,
    ) where
        Self: Sized,
        F: Fn(Option<std::path::PathBuf>) + 'static,
    {
        callback(self.screenshot_save.clone())
    }

    fn select_game_save<F>(
        &mut self,
        _title: &str,
        _default_path: Option<&str>,
        callback: F,
    ) where
        Self: Sized,
        F: Fn(Option<std::path::PathBuf>) + 'static,
    {
        callback(self.game_save.clone())
    }

    fn select_game_load<F>(
        &mut self,
        _title: &str,
        _default_path: Option<&str>,
        callback: F,
    ) where
        Self: Sized,
        F: Fn(Option<std::path::PathBuf>) + 'static,
    {
        callback(self.game_load.clone())
    }
}
