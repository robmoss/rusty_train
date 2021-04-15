/// Find optimal routes for three major companies in the final operating round
/// of the Bankruptcy Club's recorded game of 1867.
use chrono::Local;
use log::info;
use std::io::Write;

use navig18xx::game::_1867;
use navig18xx::prelude::*;

#[test]
#[ignore]
/// Run this example and write the output images to the book directory.
/// Because this example takes minutes to run, it is ignored by default.
/// Ignored tests can be run with:
///
///     cargo test [options] -- --ignored
///
/// To run all tests (normal and ignored):
///
///     cargo test [options] -- --include-ignored
///
fn run_test() -> Result<(), Box<dyn std::error::Error>> {
    let book_dir = std::path::Path::new("./book/src");
    // NOTE: this also affects where we will save/load best routes.
    assert!(std::env::set_current_dir(&book_dir).is_ok());
    main()
}

/// Run this example and write the output images to the working directory.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Default to logging all messages up to ``log::Level::Info``, using a
    // custom message format.
    let log_level = "info";
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(log_level),
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

    let hex_max_diameter = 125.0;
    let hex = Hex::new(hex_max_diameter);
    let mut game = _1867::Game::new(&hex);
    let mut example = Example::new_game(&game, hex);

    // NOTE: the major companies with tokens on the board are:
    // Blue:   Chesapeake and Ohio Railway (C&O)
    // Green:  Canadian Northern Railway (CNoR)
    // Brown:  Great Western Railway (GW)
    // Red:    Canadian Pacific Railway (CPR)
    // Beige:  National Transcontinental Railway (NTR)
    //
    // Not in play:
    // Orange: Grand Trunk Railway (GT)
    // Yellow: Intercolonial Railway of Canada (IRC)
    // Black:  New York Central Railroad (NYC)

    // Set the game phase to "8" (which has index 6, being the 7th phase).
    game.set_phase(example.get_map_mut(), 6);

    // Define the five token colours by name.
    let green = "CPR";
    let brown = "D";
    let red = "E";
    let blue = "C&O";
    let yellow = "C";

    // Place tiles and tokens.
    let tiles = vec![
        // Top-most diagonal row, starting from Sarnia.
        tile_at("87", "B18").rotate_cw(1),
        tile_at("63", "C17").token(0, blue),
        tile_at("63", "D16").token(0, green),
        tile_at("63", "E15").token(0, brown),
        tile_at("42", "F14").rotate_acw(2),
        tile_at("23", "G13").rotate_cw(1),
        tile_at("27", "H12").rotate_acw(2),
        tile_at("23", "I11").rotate_cw(1),
        tile_at("24", "J10").rotate_acw(2),
        tile_at("8", "K9").rotate_cw(1),
        // Second diagonal row, spanning Hamilton to Trois-Rivières.
        tile_at("8", "D18").rotate_acw(2),
        tile_at("623", "E17").token(0, blue).token(1, green),
        tile_at("124", "F16")
            .token(0, blue)
            .token(1, green)
            .token(2, red),
        tile_at("611", "G15").rotate_cw(1).token(0, red),
        tile_at("204", "H14").rotate_acw(1),
        tile_at("8", "I13").rotate_cw(2),
        tile_at("X8", "J12").token(0, brown),
        tile_at("31", "K11").rotate_acw(1),
        tile_at("204", "L10").rotate_acw(2),
        tile_at("57", "M9").rotate_cw(1),
        tile_at("9", "N8").rotate_cw(1),
        // Third diagonal row, Kingston to Montreal.
        tile_at("15", "I15").rotate_cw(1).token(0, yellow),
        tile_at("24", "J14").rotate_cw(1),
        tile_at("911", "K13").rotate_acw(2),
        tile_at("639", "L12")
            .token(0, red)
            .token(1, yellow)
            .token(2, brown),
        // Fourth diagonal row, connects Montreal to New England.
        tile_at("58", "M13").rotate_cw(2),
    ];
    example.place_tiles(tiles);

    // NOTE: these tiles were placed after CNoR (green) and GW (brown) ran,
    // and before C&O (blue) ran, but placing them does not affect the optimal
    // routes for CNoR and GW, so it's simplest to place them now and use the
    // same map configuration for all three companies.
    let extra_tiles = vec![
        tile_at("16", "D18").rotate_acw(3),
        tile_at("7", "C19").rotate_acw(2),
    ];
    example.place_tiles(extra_tiles);

    // Draw the entire the map.
    example.draw_map();

    // Save an image of the map prior to drawing any routes.
    save_png(&example, "1867_bc.png");

    let green_trains =
        Trains::new(vec![Train::new_5_train(), Train::new_5p5e_train()]);
    let brown_trains =
        Trains::new(vec![Train::new_5_train(), Train::new_8_train()]);
    let blue_trains =
        Trains::new(vec![Train::new_6_train(), Train::new_8_train()]);

    let companies = vec![
        (brown, brown_trains, "5-train, 8-train", 15_008, 840),
        (blue, blue_trains, "6-train, 8-train", 46_176, 900),
        (green, green_trains, "5-train, 5+5E-train", 67_948, 1130),
    ];

    // Draw the best routes for each company in turn.
    for (tok_name, trains, train_str, n, t) in companies.into_iter() {
        draw_routes(&mut example, tok_name, trains, train_str, n, t)?
    }

    Ok(())
}

fn best_routes(
    example: &Example,
    token: Token,
    trains: Trains,
    bonuses: Vec<navig18xx::route::Bonus>,
    num_paths: usize,
    net_revenue: usize,
) -> Routes {
    let path_limit = trains.path_limit();
    let criteria = Criteria {
        token,
        path_limit,
        conflict_rule: ConflictRule::TrackOrCityHex,
        route_conflict_rule: ConflictRule::TrackOnly,
    };
    let map = example.get_map();
    let start = Local::now();
    let paths = paths_for_token(&map, &criteria);
    assert_eq!(paths.len(), num_paths);
    let mid = Local::now() - start;
    let routes = trains
        .select_routes(paths, bonuses)
        .expect("Could not find optimal routes");
    let durn = Local::now() - start;
    info!(
        "Paths duration: {:02}:{:02}.{:03}",
        mid.num_minutes(),
        mid.num_seconds() % 60,
        mid.num_milliseconds() % 1000
    );
    info!(
        "Total duration: {:02}:{:02}.{:03}",
        durn.num_minutes(),
        durn.num_seconds() % 60,
        durn.num_milliseconds() % 1000
    );
    info!("${}", routes.net_revenue);
    for train_route in &routes.train_routes {
        info!("    ${}", train_route.revenue);
    }
    info!("");
    assert_eq!(routes.net_revenue, net_revenue);
    routes
}

fn draw_routes(
    example: &mut Example,
    tok_name: &str,
    trains: Trains,
    train_str: &str,
    num_paths: usize,
    net_revenue: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = example.get_map().tokens().get_token(tok_name).unwrap();
    let best =
        best_routes(&example, *token, trains, vec![], num_paths, net_revenue);

    let routes_file = format!("1867_bc_{}.json", tok_name);
    println!("Saving routes to {}", routes_file);
    let pretty = true;
    write_routes(routes_file, &best, pretty).unwrap();

    example.erase_all()?;

    // Draw the relevant portion of the map.
    example.draw_map_subset(|addr| {
        let row = addr.row();
        let col = addr.column() as usize - 'A' as usize;
        row + col >= 17
    });

    let first_rgba = (0.7, 0.1, 0.1, 1.0);
    let second_rgba = (0.1, 0.7, 0.1, 1.0);
    for (pix, tr) in best.train_routes.iter().enumerate() {
        if pix == 0 {
            example.draw_route(&tr.route, first_rgba)
        } else {
            example.draw_route(&tr.route, second_rgba)
        }
    }

    let label_text =
        format!("{}: {} = ${}", tok_name, train_str, best.net_revenue);
    let label = example
        .new_label(label_text)
        .font_family("Serif")
        .font_size(36.0)
        .weight(pango::Weight::Bold)
        .hjust(0.5)
        .vjust(0.0)
        .into_label()
        .expect("Could not create label");

    // NOTE: image coordinates are not the same as map canvas coordinates,
    // because we're only drawing a subset of the map and the full surface
    // will be cropped to the inked portion.
    // Instead, draw the label relative to a known hex address.
    let m = example.get_map().prepare_to_draw(
        "A5".parse().unwrap(),
        &example.get_hex(),
        &example.get_context(),
    );
    // Draw the text in black, then restore the transformation matrix.
    example.get_context().set_source_rgb(0.0, 0.0, 0.0);
    label.draw();
    example.get_context().set_matrix(m);

    // Save an image of the map, showing the best routes.
    let output_file = format!("1867_bc_{}.png", tok_name);
    save_png(example, output_file);

    Ok(())
}

fn save_png<S: AsRef<str>>(example: &Example, filename: S) {
    let filename = filename.as_ref();
    // NOTE: don't use a fully-transparent background (alpha = 0.0).
    // Otherwise the revenue label will not be visible in the book when using
    // a dark theme.
    let bg_rgba = Some((1.0, 1.0, 1.0, 1.0));
    let margin = 20;
    info!("Writing {} ...", filename);
    example.write_png(margin, bg_rgba, filename);
}
