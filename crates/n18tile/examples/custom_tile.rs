/// Draws a custom tile that contains a vertical phase revenue label.
use n18hex::{Delta, Direction, Hex, HexColour::*, HexFace::*, HexPosition};
use n18tile::{Label, Tile, Track};

#[test]
fn custom_tile() {
    // NOTE: when run as a test, the working directory is the n18tile root,
    // not the workspace root.
    let output_dir = "../../examples/output";
    draw_custom_tile(output_dir)
}

fn main() {
    let output_dir = "./examples/output";
    draw_custom_tile(output_dir)
}

fn draw_custom_tile(output_dir: &str) {
    let output_file = "n18tile_custom_tile_1.png";
    let output_path = std::path::Path::new(output_dir).join(output_file);
    let hex = Hex::default();
    let tile = Tile::new(
        Red,
        "Custom1",
        vec![Track::straight(UpperRight), Track::gentle_l(LowerLeft)],
        vec![],
        &hex,
    )
    .with_offboard_faces([UpperRight, LowerRight])
    .label(Label::MapLocation("Custom".to_string()), Top.to_centre(0.1))
    .label(
        Label::PhaseRevenueVert(vec![
            (Yellow, 10, false),
            (Green, 20, true),
            (Brown, 30, false),
            (Grey, 40, false),
        ]),
        HexPosition::Centre(Some(Delta::InDir(Direction::W, 0.35))),
    )
    .hide_tile_name();
    println!("Writing {} ...", output_path.display());
    tile.save_png(&hex, output_path).unwrap();
}
