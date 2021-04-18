use navig18xx::prelude::*;
use navig18xx::route::builder::{Result, RouteBuilder};
use std::io::Write;

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
    let output_dir = std::path::Path::new("./examples/output");

    let hex_max_diameter = 125.0;
    let token_a = Token::new(TokenStyle::SideArcs {
        fg: (176, 176, 176).into(),
        bg: (66, 0, 0).into(),
        text: (255, 255, 255).into(),
    });
    let tokens = vec![("A", token_a)];
    let tiles = vec![
        tile_at("5", "A1").tokens(&[(0, "A")]),
        tile_at("8", "B2").rotate_cw(4),
        tile_at("8", "C1").rotate_cw(1),
        tile_at("6", "D2").rotate_cw(2).token(0, "A"),
        tile_at("9", "A3"),
        tile_at("6", "A5").rotate_cw(5).token(0, "A"),
        tile_at("8", "B6").rotate_cw(4),
        tile_at("8", "C5").rotate_cw(1),
        tile_at("6", "D6").rotate_cw(2).token(0, "A"),
        tile_at("9", "D8"),
        tile_at("5", "A9").token(0, "A"),
        tile_at("8", "B10").rotate_cw(4),
        tile_at("8", "C9").rotate_cw(1),
        tile_at("5", "D10").rotate_cw(3).token(0, "A"),
        tile_at("9", "A11"),
        tile_at("6", "A13").rotate_cw(5).token(0, "A"),
        tile_at("8", "B14").rotate_cw(4),
        tile_at("8", "C13").rotate_cw(1),
        tile_at("6", "D14").rotate_cw(2).token(0, "A"),
    ];
    let example = Example::new(hex_max_diameter, tokens, tiles);

    let map = example.get_map();
    let route1 = RouteBuilder::from_edge(&map, "A1", HexFace::LowerRight)?
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
    let route2 = RouteBuilder::from_edge(&map, "A3", HexFace::Top)?
        .to_edge(HexFace::Bottom)?
        .to_city(0, true)?
        .into_route();

    // Find the best route for an 8-train.
    let token = map.tokens().first_token();
    let criteria = Criteria {
        token,
        path_limit: None,
        conflict_rule: ConflictRule::TrackOrCityHex,
        route_conflict_rule: ConflictRule::TrackOnly,
    };
    let paths = paths_for_token(&map, &criteria);
    // TODO: need a more flexible way to define train types.
    let trains = Trains::new(vec![Train::new_8_train()]);
    let best_routes = trains.select_routes(paths, vec![]);
    let best_route = &best_routes.unwrap().train_routes[0].route;

    example.draw_map();
    example.draw_route(&best_route, (0.1, 0.7, 0.1, 1.0));
    example.draw_route(&route1, (0.7, 0.1, 0.1, 1.0));
    example.draw_route(&route2, (0.1, 0.1, 0.7, 1.0));

    // NOTE: use Pango to draw a large label above the map.
    let ctx = example.get_context();
    let centre_label = false;
    let hjust = if centre_label { 0.5 } else { 0.0 };
    let label = example
        .new_label("navig18xx")
        .font_family("Serif")
        .font_size(36.0)
        .weight(pango::Weight::Bold)
        .hjust(hjust)
        .into_label()
        .expect("Could not create label");
    let label_height = label.dims().1;
    ctx.set_source_rgb(0.0, 0.0, 0.0);
    if centre_label {
        let image_width = example.content_size().0;
        label.draw_at(0.5 * image_width, -2.0 * label_height as f64);
    } else {
        label.draw_at(0.0, -2.0 * label_height as f64);
    }

    let bg_rgba = Some((1.0, 1.0, 1.0, 1.0));
    let margin = 20;
    example.write_png(margin, bg_rgba, output_dir.join("example_routes.png"));
    example.write_svg(margin, bg_rgba, output_dir.join("example_routes.svg"));
    example.write_pdf(margin, bg_rgba, output_dir.join("example_routes.pdf"));

    Ok(())
}
