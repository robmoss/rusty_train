//! Draws a saved game and writes the image to a PDF, PNG, or SVG file.
//!
//! # Command-line usage
//!
//! ```text
//! cargo run --example draw_game -- --pdf saved.game output.pdf
//! cargo run --example draw_game -- --png saved.game output.png
//! cargo run --example draw_game -- --svg saved.game output.svg
//! ```
//!
//! The default output filename is the input filename with the appropriate
//! extension (pdf, png, or svg).
//!

use std::path::PathBuf;

use navig18xx::prelude::{image_size, Game, Hex, ImageFormat, Map};

/// Program settings, which can be overridden by command-line arguments.
pub struct Settings {
    /// The output image format.
    pub format: ImageFormat,
    /// The map hexagon size.
    pub hex_size: f64,
    /// The input game state file.
    pub input_file: Option<PathBuf>,
    /// The output image file.
    pub output_file: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            hex_size: 125.0,
            format: ImageFormat::Png,
            input_file: None,
            output_file: None,
        }
    }
}

impl Settings {
    /// Returns the program settings after parsing any command-line arguments.
    ///
    /// Returns `None` if there were invalid or missing arguments.
    pub fn try_from_args() -> Option<Self> {
        let mut settings = Settings::default();
        let mut parse_options = true;
        for arg in std::env::args().skip(1) {
            if parse_options {
                if !arg.starts_with('-') {
                    // Stop parsing options.
                    parse_options = false;
                } else if arg == "--" {
                    // Stop parsing options, and skip this argument.
                    parse_options = false;
                    continue;
                } else {
                    match arg.as_str() {
                        "--pdf" => settings.format = ImageFormat::Pdf,
                        "--png" => settings.format = ImageFormat::Png,
                        "--svg" => settings.format = ImageFormat::Svg,
                        _ => return None,
                    }
                    continue;
                }
            };
            if settings.input_file.is_none() {
                settings.input_file = Some(PathBuf::from(arg));
            } else if settings.output_file.is_none() {
                settings.output_file = Some(PathBuf::from(arg));
            } else {
                return None;
            }
        }

        // Check that we received the necessary arguments.
        if let Some(ref input_file) = settings.input_file {
            if settings.output_file.is_none() {
                let ext = settings.format.extension();
                settings.output_file = Some(input_file.with_extension(ext))
            }
        } else {
            return None;
        }

        Some(settings)
    }
}

/// Draws the game map on the provided context.
pub fn draw(map: &Map, hex: &Hex, ctx: &cairo::Context) {
    let mut hex_iter = map.hex_iter(hex, ctx);
    navig18xx::brush::draw_map(hex, ctx, &mut hex_iter);
}

pub fn main() {
    let settings = Settings::try_from_args().expect("Invalid arguments");
    let hex = Hex::new(settings.hex_size);
    let input = settings.input_file.unwrap();
    let output = settings.output_file.unwrap();

    // Index the available games by name.
    let mut games: std::collections::BTreeMap<String, Box<dyn Game>> =
        vec![Box::new(navig18xx::game::new_1867()) as Box<dyn Game>]
            .into_iter()
            .map(|game| (game.name().to_string(), game))
            .collect();

    // Load the game state from disk.
    println!("Reading {} ...", input.to_str().unwrap());
    let game_state = navig18xx::io::read_game_state(&input)
        .expect("Could not read game file");

    // Identify the appropriate game for this game state, and load the map.
    let game = games
        .get_mut(&game_state.game)
        .expect("No matching game for game file");
    let map = game.load(&hex, game_state).expect("Could not load map");

    // Determine the appropriate image size, and save the image to disk.
    let (width, height) = image_size(|ctx| draw(&map, &hex, ctx))
        .expect("Could not determine image size");
    println!("Writing {} ...", output.to_str().unwrap());
    settings
        .format
        .save_image(width, height, |ctx| draw(&map, &hex, ctx), &output)
        .expect("Could not write output image")
}
