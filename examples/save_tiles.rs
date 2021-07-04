//! Save collections of tiles to JSON files.
use navig18xx::prelude::*;

mod output;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_save_tiles() -> Result {
    let output_dir = output::Dir::Examples;
    save_tiles(&output_dir)
}

fn main() -> Result {
    let output_dir = output::Dir::Root;
    save_tiles(&output_dir)
}

fn save_tiles(output_dir: &output::Dir) -> Result {
    let pretty_json = true;
    let hex_max_diameter = 125.0;
    let hex = Hex::new(hex_max_diameter);
    let tiles = tile_catalogue(&hex);
    write_tiles(output_dir.join("tile_catalogue.json"), &tiles, pretty_json)?;

    let game = navig18xx::game::_1867::Game::new(&hex);
    let tiles = game.all_tiles();
    write_tiles(output_dir.join("tile_1867.json"), tiles, pretty_json)?;

    Ok(())
}
