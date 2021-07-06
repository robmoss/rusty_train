/// Display all tiles from the tile catalogue in a rectangular map.
use navig18xx::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hex_max_diameter = 125.0;
    let num_rows = 6;
    let num_cols = 14;
    let margin = 10;
    let bg_rgba = Some((1.0, 1.0, 1.0, 1.0));
    let png_file = "tile_catalogue.png";

    let hex = Hex::new(hex_max_diameter);
    let tiles = tile_catalogue(&hex);
    let example = place_tiles(hex, &tiles, num_rows, num_cols);
    example.draw_map();
    example.write_png(margin, bg_rgba, png_file);

    let png_file = "tile_1867.png";
    let num_rows = 8;
    let num_cols = 16;

    let hex = Hex::new(hex_max_diameter);
    let game = navig18xx::game::_1867::Game::new(&hex);
    // NOTE: this method currently returns special tiles too.
    let tiles = game.player_tiles();
    let example = place_tiles(hex, &tiles, num_rows, num_cols);
    example.draw_map();
    example.write_png(margin, bg_rgba, png_file);

    Ok(())
}

fn place_tiles(
    hex: Hex,
    tiles: &[Tile],
    rows: usize,
    cols: usize,
) -> Example {
    // Build an iterator over tile names for the tile catalogue.
    let tile_names = tiles.iter().map(|t| &t.name).cycle();

    // Build an iterator over the map hexes.
    let mut tile_addrs: Vec<String> = vec![];
    let rows: Vec<usize> = (0..rows).collect();
    let cols: Vec<usize> = (0..cols).collect();
    for row in &rows {
        for col in &cols {
            let addr = HexAddress::new(*row, *col);
            tile_addrs.push(format!("{}", addr))
        }
    }

    // Combine the two iterators to place a tile at each map hex.
    let placed_tiles: Vec<_> = tile_addrs
        .iter()
        .zip(tile_names)
        .map(|(addr, name)| tile_at(name, addr))
        .collect();

    let tokens: Vec<(String, _)> = vec![];
    Example::new_catalogue(hex, tokens, placed_tiles, tiles.to_vec())
}
