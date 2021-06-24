//! Read collections of tiles from JSON files and draw them as PNG files.
use navig18xx::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hex_max_diameter = 125.0;
    let margin = 10;
    let bg_rgba = Some((1.0, 1.0, 1.0, 1.0));

    let mut rows: usize = 6;
    let mut cols: usize = 14;
    let mut json_files: Vec<String> = vec![];

    // Skip the first argument, which is typically the path to this
    // executable, but could conceivably contain anything.
    let mut args = std::env::args();
    args.next();

    loop {
        if let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_usage();
                    return Ok(());
                }
                "-r" => {
                    if let Some(row_str) = args.next() {
                        rows = row_str.parse::<usize>()?
                    } else {
                        panic!("Missing argument for {}", arg)
                    }
                }
                "-c" => {
                    if let Some(row_str) = args.next() {
                        cols = row_str.parse::<usize>()?
                    } else {
                        panic!("Missing argument for {}", arg)
                    }
                }
                _ => json_files.push(arg),
            }
        } else {
            break;
        }
    }

    if json_files.len() == 0 {
        println!("ERROR: No input files given");
        print_usage();
        return Ok(());
    }

    for json_file in &json_files {
        let hex = Hex::new(hex_max_diameter);
        println!("Reading {} ...", json_file);
        let tiles = read_tiles(json_file, &hex)?;

        let example = place_tiles(hex, &tiles, rows, cols);
        example.draw_map();

        let png_file = std::path::Path::new(json_file).with_extension("png");
        println!("Writing {} ...", png_file.to_str().unwrap());
        example.write_png(margin, bg_rgba, png_file);
    }

    Ok(())
}

fn print_usage() {
    println!("");
    println!("draw_tiles [-c COLS] [-r ROWS] JSON_FILES");
    println!("");
    println!("    -c COLS       The number of tile columns");
    println!("    -r ROWS       The number of tile rows");
    println!("    JSON_FILES    The tile JSON file(s) to draw");
    println!("");
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
    let example =
        Example::new_catalogue(hex, tokens, placed_tiles, tiles.to_vec());
    example
}
