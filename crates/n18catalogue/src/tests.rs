use super::*;

#[test]
fn compare_tile_fns() {
    let catalogue_tiles = tile_catalogue();
    let hex: Hex = Hex::default();
    let kind_tiles: Vec<Tile> =
        Kind::iter().map(|kind| kind.build(&hex)).collect();
    for tile in &kind_tiles {
        assert!(catalogue_tiles.iter().any(|t| t == tile))
    }
    assert_eq!(catalogue_tiles, kind_tiles);
    let bcat = Builder::all_tiles().build();
    let bcat_tiles: Vec<Tile> = bcat.tile_iter().cloned().collect();
    assert_eq!(catalogue_tiles, bcat_tiles);
}
