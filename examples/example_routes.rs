use navig18xx::hex::theme::AlignH;
use navig18xx::prelude::*;
use navig18xx::route::builder::{Result, RouteBuilder};
use std::io::Write;

mod output;
use output::Dir;

#[test]
fn run_test() -> Result<()> {
    main()
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .format(|buf, record| {
        writeln!(
            buf,
            "{} [{}] {}",
            chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
            record.level(),
            record.args()
        )
    })
    .init();

    // Specify where to save the output images.
    let output_dir = Dir::Examples;

    let hex_max_diameter = 125.0;
    let token_a = Token::new(TokenStyle::SideArcs {
        fg: (176, 176, 176).into(),
        bg: (66, 0, 0).into(),
        text: Colour::WHITE,
    });
    let name_a = "A";
    let tokens = vec![(name_a, token_a)];
    let tiles = vec![
        tile_at("5", "A1").tokens(&[(0, name_a)]),
        tile_at("8", "B2").rotate_cw(4),
        tile_at("8", "C1").rotate_cw(1),
        tile_at("6", "D2").rotate_cw(2).token(0, name_a),
        tile_at("9", "A3"),
        tile_at("6", "A5").rotate_cw(5).token(0, name_a),
        tile_at("8", "B6").rotate_cw(4),
        tile_at("8", "C5").rotate_cw(1),
        tile_at("6", "D6").rotate_cw(2).token(0, name_a),
        tile_at("9", "D8"),
        tile_at("5", "A9").token(0, name_a),
        tile_at("8", "B10").rotate_cw(4),
        tile_at("8", "C9").rotate_cw(1),
        tile_at("5", "D10").rotate_cw(3).token(0, name_a),
        tile_at("9", "A11"),
        tile_at("6", "A13").rotate_cw(5).token(0, name_a),
        tile_at("8", "B14").rotate_cw(4),
        tile_at("8", "C13").rotate_cw(1),
        tile_at("6", "D14").rotate_cw(2).token(0, name_a),
    ];
    let coords = Coordinates {
        orientation: Orientation::FlatTop,
        letters: Letters::AsColumns,
        first_row: FirstRow::OddColumns,
    };
    let example = Example::new(hex_max_diameter, tokens, tiles, coords);

    let map = example.map();
    let a1 = coords.parse("A1").unwrap();
    let a3 = coords.parse("A3").unwrap();

    let route1 = RouteBuilder::from_edge(map, a1, HexFace::LowerRight)?
        .to_city(0, true)?
        .to_edge(HexFace::Bottom)?
        .to_edge(HexFace::Bottom)?
        .to_city(0, true)?
        .to_edge(HexFace::LowerRight)?
        .to_edge(HexFace::UpperRight)?
        .to_edge(HexFace::LowerRight)?
        .to_city(0, false)?
        .to_edge(HexFace::Bottom)?
        .into_route();
    let route2 = RouteBuilder::from_edge(map, a3, HexFace::Top)?
        .to_edge(HexFace::Bottom)?
        .to_city(0, true)?
        .into_route();

    // Find the best route for an 8-train.
    let token = map.token(name_a);
    let criteria = Criteria {
        token,
        path_limit: None,
        conflict_rule: ConflictRule::TrackOrCityHex,
        route_conflict_rule: ConflictRule::TrackOnly,
    };
    let paths = paths_for_token(map, &criteria);
    let trains = Trains::new(vec![TrainType::SkipTowns.with_max_stops(8)]);
    let best_routes = trains.select_routes(paths, vec![]);
    let best_route = &best_routes.unwrap().train_routes[0].route;

    example.draw_map();
    example.draw_route(best_route, example.theme().nth_highlight_colour(1));
    example.draw_route(&route1, example.theme().nth_highlight_colour(0));
    example.draw_route(&route2, example.theme().nth_highlight_colour(2));

    // NOTE: use Pango to draw a large label above the map.
    let ctx = example.context();
    let centre_label = false;
    let horiz = if centre_label {
        AlignH::Centre
    } else {
        AlignH::Left
    };
    let labeller = example
        .text_style()
        .font_serif()
        .font_size(36.0)
        .bold()
        .halign(horiz)
        .labeller(ctx, example.hex());
    let label_text = "navig18xx";
    let label_height = labeller.size(label_text).height;
    let coords = if centre_label {
        let image_width = example.content_size().0;
        (0.5 * image_width, -2.0 * label_height)
    } else {
        (0.0, -2.0 * label_height)
    };
    labeller.draw(label_text, coords.into());

    let bg_rgba = Some(Colour::WHITE);
    let margin = 20;
    example.write_png(margin, bg_rgba, output_dir.join("example_routes.png"));
    example.write_svg(margin, bg_rgba, output_dir.join("example_routes.svg"));
    example.write_pdf(margin, bg_rgba, output_dir.join("example_routes.pdf"));

    Ok(())
}
