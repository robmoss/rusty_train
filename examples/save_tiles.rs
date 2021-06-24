//! Save collections of tiles to JSON files.
use navig18xx::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pretty_json = true;
    let hex_max_diameter = 125.0;
    let hex = Hex::new(hex_max_diameter);
    let tiles = tile_catalogue(&hex);
    write_tiles("tile_catalogue.json", &tiles, pretty_json)?;

    let game = navig18xx::game::_1867::Game::new(&hex);
    let tiles = game.player_tiles();
    write_tiles("tile_1867.json", tiles, pretty_json)?;

    Ok(())
}
