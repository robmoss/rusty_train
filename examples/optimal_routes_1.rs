use navig18xx::prelude::*;
mod output;
use output::Dir;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_optimal_routes_1() -> Result {
    let output_dir = Dir::DevGuide;
    optimal_routes_1(&output_dir)
}

fn main() -> Result {
    let output_dir = Dir::DevGuide;
    optimal_routes_1(&output_dir)
}

fn optimal_routes_1(output_dir: &Dir) -> Result {
    let hex_max_diameter = 125.0;
    let token_a = Token::new(TokenStyle::SideArcs {
        fg: (176, 176, 176).into(),
        bg: (66, 0, 0).into(),
        text: (255, 255, 255).into(),
    });
    let tokens = vec![("A", token_a)];

    // $70 --- $100 (token) --- $100 --- $50
    let tiles = vec![
        tile_at("X6", "B2"),
        tile_at("124", "C3").token(0, "A").rotate_acw(1),
        tile_at("124", "C1").rotate_cw(2),
        tile_at("801", "D2"),
    ];
    let mut example = Example::new(hex_max_diameter, tokens, tiles);

    // The different train combinations.
    let (t8, t2p2) = (Train::new_8_train(), Train::new_2p2_train());
    let trains_8 = Trains::new(vec![t8]);
    let trains_2p2 = Trains::new(vec![t2p2]);
    let trains_both = Trains::new(vec![t2p2, t8]);
    let combinations =
        [("8", trains_8), ("2p2", trains_2p2), ("both", trains_both)];

    // Find all available routes, ignoring limits on the number of stops.
    let map = example.get_map();
    let token = map.tokens().first_token();
    let criteria = Criteria {
        token,
        path_limit: None,
        conflict_rule: ConflictRule::TrackOrCityHex,
        route_conflict_rule: ConflictRule::TrackOnly,
    };
    let paths = paths_for_token(&map, &criteria);
    let bonuses = vec![];

    // Background and route colours, image margins.
    let t8_rgba = (0.7, 0.1, 0.1, 1.0);
    let t2p2_rgba = (0.1, 0.7, 0.1, 1.0);
    let bg_rgba = Some((1.0, 1.0, 1.0, 0.0));
    let margin = 20;

    // Save the map (without any routes) to disk.
    example.draw_map();
    example.write_png(margin, bg_rgba, output_dir.join("opt_r1.png"));

    for (suffix, trains) in &combinations {
        // Find the best route(s) for this train combination.
        let best_routes = trains
            .select_routes(paths.clone(), bonuses.clone())
            .expect(&format!("Could not find optimal routes for {}", suffix));

        // Clear the image buffer.
        example
            .erase_all()
            .expect("Could not erase example content");

        // Draw each of the routes operated by the company.
        example.draw_map();
        for tr in &best_routes.train_routes {
            if tr.train == t8 {
                example.draw_route(&tr.route, t8_rgba);
            } else {
                example.draw_route(&tr.route, t2p2_rgba);
            }
        }

        // Save the image to disk.
        let basename = format!("opt_r1_{}.png", suffix);
        example.write_png(margin, bg_rgba, output_dir.join(basename));
    }

    Ok(())
}
