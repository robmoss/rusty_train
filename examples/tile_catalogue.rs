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

    // Build an iterator over tile names for the tile catalogue.
    let tiles = tile_catalogue(&hex);
    let tokens: Vec<(String, _)> = vec![];
    let tile_names = tiles.iter().map(|t| &t.name).cycle();

    // Build an iterator over the map hexes.
    let mut tile_addrs: Vec<String> = vec![];
    let rows: Vec<usize> = (0..num_rows).collect();
    let cols: Vec<usize> = (0..num_cols).collect();
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

    // Draw the resulting map and save it as a PNG file.
    let example = Example::new(hex, tokens, placed_tiles);
    example.draw_map();
    example.write_png(margin, bg_rgba, png_file);
    Ok(())
}
