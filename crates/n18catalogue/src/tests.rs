use super::*;

#[test]
fn compare_tile_fns() {
    let good_tiles = tile_catalogue();
    let hex: Hex = Hex::default();
    let new_tiles: Vec<Tile> = tiles::all_tile_fns()
        .iter()
        .map(|(name, tile_fn)| tile_fn(&hex, name))
        .collect();
    for tile in &new_tiles {
        assert!(good_tiles.iter().any(|t| t == tile))
    }
    assert_eq!(good_tiles, new_tiles);
    let cat = Builder::all_tiles().build();
    let cat_tiles: Vec<Tile> = cat.tile_iter().cloned().collect();
    assert_eq!(good_tiles, cat_tiles);
}
