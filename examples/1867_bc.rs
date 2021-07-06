/// Find optimal routes for three major companies in the final operating round
/// of the Bankruptcy Club's recorded game of 1867.
///
/// Run this as a test case to calculate the optimal routes and ensure that
/// they match the cached results:
///
///     cargo test --release 1867_bc -- --ignored
///
/// Run this as an example to use the cached results (if they exist) and
/// update the output images in the book directory:
///
///     cargo run --release --example 1867_bc
///
use chrono::Local;
use log::info;
use std::io::Write;
use std::path::Path;

use navig18xx::game::_1867;
use navig18xx::prelude::*;

mod output;
use output::Dir;

/// The state of the 1867 game.
pub struct GameState {
    example: Example,
    companies: Vec<CompanyInfo>,
}

/// The details of each company and their optimal routes.
pub struct CompanyInfo {
    token_name: &'static str,
    trains: Trains,
    train_desc: &'static str,
    num_paths: usize,
    net_revenue: usize,
}

#[test]
#[ignore]
/// Run this example and write the output images to the working directory.
/// This will always calculate the optimal routes, and ensure that they are
/// identical to the cached routes (if they exist).
///
/// Because this example takes minutes to run, it is ignored by default.
/// Ignored tests can be run with:
///
///     cargo test [options] -- --ignored
///
fn test_1867_bc() -> Result<(), Box<dyn std::error::Error>> {
    let use_cached_routes = false;
    save_1867_bc_routes(use_cached_routes)
}

/// Run this example and write the output images to the book directory.
/// This will use the cached routes, if they exist.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let use_cached_routes = true;
    save_1867_bc_routes(use_cached_routes)
}

/// Default to logging all messages up to ``log::Level::Info``.
fn init_logging() {
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
}

fn save_1867_bc_routes(
    use_cached_routes: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let image_dir = Dir::BookRoot;
    let json_dir = Dir::Examples;

    init_logging();

    let mut state = game_state();

    // Save an image of the map prior to drawing any routes.
    state.example.draw_map();
    let out_file = image_dir.join("1867_bc.png");
    save_png(&state.example, &out_file);

    // Draw the best routes for each company in turn.
    for company in &state.companies {
        // Determine the output file name for the best routes.
        let routes_basename = format!("1867_bc_{}.json", company.token_name);
        let routes_file = json_dir.join(routes_basename);
        // Determine the output file name for the best routes image.
        let image_basename = format!("1867_bc_{}.png", company.token_name);
        let image_file = image_dir.join(image_basename);

        // Load the cached routes, if they exist.
        let cached_opt = read_routes(&routes_file).ok();
        // Determine whether to calculate and save the best routes.
        let (routes, save_routes) = if let Some(routes) = cached_opt {
            info!("Reading {}", (&routes_file).to_str().unwrap());
            if use_cached_routes {
                // Use the cached routes, no need to save them.
                info!("Using cached routes for {}", company.token_name);
                (routes, false)
            } else {
                info!("Calculating best routes for {}", company.token_name);
                let new_routes = best_routes(&state.example, &company);
                if new_routes == routes {
                    // The calculated routes match the cached routes, no need
                    // to save them.
                    (routes, false)
                } else {
                    // The calculated routes differ from the cached routes.
                    // Save the calculated routes and fail the test.
                    info!(
                        "Saving routes to {}",
                        (&routes_file).to_str().unwrap()
                    );
                    let pretty = true;
                    write_routes(routes_file, &new_routes, pretty).unwrap();
                    panic!("Calculated routes differ from the cached routes")
                }
            }
        } else {
            // Calculate the best routes and save the results.
            (best_routes(&state.example, &company), true)
        };

        // Draw the best routes and save the image to disk.
        state.example.erase_all()?;
        draw_routes(&mut state.example, &company, &routes)?;
        save_png(&state.example, image_file);

        // If the best routes were calculated, save them to disk.
        if save_routes {
            info!("Saving routes to {}", (&routes_file).to_str().unwrap());
            let pretty = true;
            write_routes(routes_file, &routes, pretty).unwrap();
        }
    }

    Ok(())
}

fn game_state() -> GameState {
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

    // Set the game phase to "8".
    game.set_phase_name(example.get_map_mut(), "8");

    // Define the five tokens by company name.
    let cnr = "CNR";
    let gw = "GW";
    let cno = "C&O";
    let cpr = "CPR";
    let ntr = "NTR";

    // Place tiles and tokens.
    let tiles = vec![
        // Top-most diagonal row, starting from Sarnia.
        tile_at("87", "B18").rotate_cw(1),
        tile_at("63", "C17").token(0, cno),
        tile_at("63", "D16").token(0, cnr),
        tile_at("63", "E15").token(0, gw),
        tile_at("42", "F14").rotate_acw(2),
        tile_at("23", "G13").rotate_cw(1),
        tile_at("27", "H12").rotate_acw(2),
        tile_at("23", "I11").rotate_cw(1),
        tile_at("24", "J10").rotate_acw(2),
        tile_at("8", "K9").rotate_cw(1),
        // Second diagonal row, spanning Hamilton to Trois-Rivières.
        tile_at("8", "D18").rotate_acw(2),
        tile_at("623", "E17").token(0, cno).token(1, cnr),
        tile_at("124", "F16")
            .token(0, cno)
            .token(1, cnr)
            .token(2, cpr),
        tile_at("611", "G15").rotate_cw(1).token(0, cpr),
        tile_at("204", "H14").rotate_acw(1),
        tile_at("8", "I13").rotate_cw(2),
        tile_at("X8", "J12").token(0, gw),
        tile_at("31", "K11").rotate_acw(1),
        tile_at("204", "L10").rotate_acw(2),
        tile_at("57", "M9").rotate_cw(1),
        tile_at("9", "N8").rotate_cw(1),
        // Third diagonal row, Kingston to Montreal.
        tile_at("15", "I15").rotate_cw(1).token(0, ntr),
        tile_at("24", "J14").rotate_cw(1),
        tile_at("911", "K13").rotate_acw(2),
        tile_at("639", "L12")
            .token(0, cpr)
            .token(1, ntr)
            .token(2, gw),
        // Fourth diagonal row, connects Montreal to New England.
        tile_at("58", "M13").rotate_cw(2),
    ];
    example.place_tiles(tiles);

    // NOTE: these tiles were placed after CNR and GW ran, and before C&O ran,
    // but placing them does not affect the optimal routes for CNR and GW, so
    // it's simplest to place them now and use the same map configuration for
    // all three companies.
    let extra_tiles = vec![
        tile_at("16", "D18").rotate_acw(3),
        tile_at("7", "C19").rotate_acw(2),
    ];
    example.place_tiles(extra_tiles);

    let cnr_trains =
        Trains::new(vec![*game.get_train("5"), *game.get_train("5+5E")]);
    let gw_trains =
        Trains::new(vec![*game.get_train("5"), *game.get_train("8")]);
    let cno_trains =
        Trains::new(vec![*game.get_train("6"), *game.get_train("8")]);

    let companies = vec![
        CompanyInfo {
            token_name: gw,
            trains: gw_trains,
            train_desc: "5-train, 8-train",
            num_paths: 15_008,
            net_revenue: 840,
        },
        CompanyInfo {
            token_name: cno,
            trains: cno_trains,
            train_desc: "6-train, 8-train",
            num_paths: 46_176,
            net_revenue: 900,
        },
        CompanyInfo {
            token_name: cnr,
            trains: cnr_trains,
            train_desc: "5-train, 5+5E-train",
            num_paths: 67_948,
            net_revenue: 1130,
        },
    ];

    GameState { example, companies }
}

fn best_routes(example: &Example, company: &CompanyInfo) -> Routes {
    let bonuses = vec![];
    let token = example
        .get_map()
        .tokens()
        .get_token(company.token_name)
        .unwrap();
    let token = *token;
    let path_limit = company.trains.path_limit();
    let criteria = Criteria {
        token,
        path_limit,
        conflict_rule: ConflictRule::TrackOrCityHex,
        route_conflict_rule: ConflictRule::TrackOnly,
    };
    let map = example.get_map();
    let start = Local::now();
    let paths = paths_for_token(&map, &criteria);
    assert_eq!(paths.len(), company.num_paths);
    let mid = Local::now() - start;
    let routes = company
        .trains
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
    assert_eq!(routes.net_revenue, company.net_revenue);
    routes
}

fn draw_routes(
    example: &mut Example,
    company: &CompanyInfo,
    routes: &Routes,
) -> Result<(), Box<dyn std::error::Error>> {
    // Draw the relevant portion of the map.
    example.draw_map_subset(|addr| {
        let row = addr.row();
        let col = addr.column() as usize - 'A' as usize;
        row + col >= 17
    });

    let first_rgba = (0.7, 0.1, 0.1, 1.0);
    let second_rgba = (0.1, 0.7, 0.1, 1.0);
    for (pix, tr) in routes.train_routes.iter().enumerate() {
        if pix == 0 {
            example.draw_route(&tr.route, first_rgba)
        } else {
            example.draw_route(&tr.route, second_rgba)
        }
    }

    let label_text = format!(
        "{}: {} = ${}",
        company.token_name, company.train_desc, routes.net_revenue
    );
    let label = example
        .new_label(label_text)
        .font_family("Serif")
        .font_size(36.0)
        .bold()
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

    Ok(())
}

fn save_png<S: AsRef<Path>>(example: &Example, filename: S) {
    let filename = filename.as_ref();
    // NOTE: don't use a fully-transparent background (alpha = 0.0).
    // Otherwise the revenue label will not be visible in the book when using
    // a dark theme.
    let bg_rgba = Some((1.0, 1.0, 1.0, 1.0));
    let margin = 20;
    info!("Writing {} ...", filename.to_str().unwrap());
    example.write_png(margin, bg_rgba, filename);
}
