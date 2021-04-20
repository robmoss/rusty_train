use navig18xx::prelude::*;

mod output;
use output::Dir;

fn main() {
    // Specify where to save the output images.
    let output_dir = Dir::BookRoot;

    let hex_max_diameter = 125.0;
    let hex = Hex::new(hex_max_diameter);

    let tile_x5 = Tile::new(
        HexColour::Brown,
        "X5",
        vec![
            Track::straight(HexFace::Top).with_span(0.0, 0.1),
            Track::straight(HexFace::Top)
                .with_span(0.1, 1.0)
                .with_clip(0.3625, 0.75),
            Track::mid(HexFace::UpperLeft),
            Track::mid(HexFace::LowerLeft),
            Track::mid(HexFace::LowerRight),
            Track::mid(HexFace::UpperRight),
        ],
        vec![
            City::single_at_face(70, &HexFace::Top),
            City::double(70).nudge(Direction::S, 0.1),
        ],
        &hex,
    )
    .label(Label::City("M".to_string()), HexCorner::BottomLeft)
    .label(Label::Revenue(0), HexCorner::Left.to_centre(0.1));

    tile_x5
        .save_svg(&hex, output_dir.join("tile_x5.svg"))
        .expect("Could not save tile X5 as an SVG");
}
