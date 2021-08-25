/// Find optimal routes for a single company in different phases of 1861,
/// where the bonus for running between Moscow and Ekaterinburg is a deciding
/// factor.
use navig18xx::prelude::*;
use std::io::Write;

/// The different phases for which to find the optimal routes, and the
/// expected optimal revenue for each phase.
const PHASES: [(&str, (usize, usize)); 4] = [
    ("2", (70, 150)),
    ("3", (90, 170)),
    ("5", (120, 190)),
    ("6", (170, 240)),
];

/// Returns the hex where Astrakhan is located.
pub fn astrakhan() -> HexAddress {
    (10, 12).into()
}

/// Returns the hex where Moscow is located.
pub fn moscow() -> HexAddress {
    (4, 7).into()
}

/// Returns the hex where Nizhnii Novgorod is located.
pub fn nizhnii_novgorod() -> HexAddress {
    (4, 10).into()
}

/// Returns the hex where St Petersburg is located.
pub fn st_petersburg() -> HexAddress {
    (1, 4).into()
}

/// Returns the hex where Vilnius is located.
pub fn vilnius() -> HexAddress {
    (4, 1).into()
}

/// Run this example, and find the optimal routes for different game phases.
#[test]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    for &(phase, revenues) in &PHASES {
        find_routes_for_phase(phase, "2", revenues.0)?;
        find_routes_for_phase(phase, "3", revenues.1)?;
    }
    // NOTE: also test the 2+2 train when it can run direct between Moscow and
    // Ekaterinburg.
    find_routes_for_phase("7", "2+2", 360)?;
    Ok(())
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

fn find_routes_for_phase(
    phase_name: &str,
    train_name: &str,
    expected_revenue: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Specify where to save the output images.
    let output_dir = std::path::Path::new("./tests/output");

    let hex = Hex::default();
    let mut game = navig18xx::game::new_1861();
    let mut map = game.create_map(&hex);
    game.set_phase_name(&mut map, phase_name);

    // Place tiles as appropriate for the chosen game phase.
    let phase_num = 2 + game.phase_ix();
    place_yellow_tiles(&mut map);
    if phase_num >= 3 {
        place_green_tiles(&mut map);
    }
    if phase_num >= 5 {
        place_brown_tiles(&mut map);
    }
    if phase_num >= 6 {
        place_grey_tiles(&mut map);
    }
    if phase_num >= 7 {
        place_skip_nizhnii_tiles(&mut map);
    }

    // Place a KB token in Moscow on the upper-right token space (yellow and
    // green phases) or a central token space (brown and grey phases).
    let token = map.token("KB");
    let token_spaces = map.tile_at(moscow()).unwrap().token_spaces();
    let hex_state = map.hex_state_mut(moscow()).unwrap();
    hex_state.set_token_at(&token_spaces[2], token);

    // Run the train(s) and identify the optimal revenue.
    let trains = Trains::new(vec![*game.train(train_name)]);
    let bonus_flags = game.bonus_options().iter().map(|_| false).collect();
    let routes = game.best_routes(&map, token, &trains, bonus_flags).unwrap();
    println!("Phase {}: ${}", phase_name, routes.net_revenue);
    assert_eq!(routes.net_revenue, expected_revenue);

    // Draw the map and highlight the optimal routes.
    let rec_surf =
        cairo::RecordingSurface::create(cairo::Content::ColorAlpha, None)
            .expect("Can't create recording surface");
    let rec_ctx =
        cairo::Context::new(&rec_surf).expect("Can't create cairo::Context");
    let mut hex_iter = map.hex_iter(&hex, &rec_ctx);
    draw_map(&hex, &rec_ctx, &mut hex_iter);
    highlight_routes(&hex, &rec_ctx, &map, &routes.routes(), |_| {
        Colour::from((159, 0, 0))
    });

    // Add a revenue label two rows above Astrakhan.
    let labeller = navig18xx::hex::theme::Text::new()
        .font_size(36.0)
        .halign_left()
        .valign_middle()
        .font_serif()
        .bold()
        .labeller(&rec_ctx, &hex);
    let addr = astrakhan()
        .adjacent_unchecked(HexFace::Top)
        .adjacent_unchecked(HexFace::Top);
    let m = map.prepare_to_draw(addr, &hex, &rec_ctx);
    labeller.draw(
        &format!("{}-train: ${}", train_name, routes.net_revenue),
        (0.0, 0.0).into(),
    );
    rec_ctx.set_matrix(m);

    // Create an appropriately-sized image surface.
    let (x0, y0, width, height) = rec_surf.ink_extents();
    let margin = 10.0;
    let surf = cairo::ImageSurface::create(
        cairo::Format::ARgb32,
        (width + 2.0 * margin) as i32,
        (height + 2.0 * margin) as i32,
    )
    .expect("Can't create surface");
    let ctx =
        cairo::Context::new(&surf).expect("Can't create cairo::Context");

    // Copy the map to the image surface, and save the image to disk.
    ctx.set_source_surface(&rec_surf, margin - x0, margin - y0)
        .unwrap();
    ctx.paint().unwrap();
    let basename =
        format!("1861_ekat_phase_{}_{}-train.png", phase_name, train_name);
    let filename = output_dir.join(basename);
    println!("Writing {} ...", filename.to_str().unwrap());
    let mut file =
        std::fs::File::create(filename).expect("Can't create output file");
    surf.write_to_png(&mut file)
        .expect("Can't write output file");

    Ok(())
}

/// Connects Moscow to Ekaterinburg (through Nizhnii Novgorod), and connects
/// St Petersburg to Poland (through Vilnius).
fn place_yellow_tiles(map: &mut Map) {
    use HexFace::*;
    use RotateCW::*;

    // Connect St Petersburg to Poland.
    map.place_tile(vilnius(), "4", One);

    // Connect Moscow to Ekaterinburg.
    moscow()
        .move_and_do(UpperRight, |&addr| {
            let _ = map.place_tile(addr, "8", One);
        })
        .move_and_do(LowerRight, |&addr| {
            let _ = map.place_tile(addr, "8", Four);
        })
        .adjacent_unchecked(UpperRight)
        .move_and_do(LowerRight, |&addr| {
            let _ = map.place_tile(addr, "9", Two);
        })
        .move_and_do(LowerRight, |&addr| {
            let _ = map.place_tile(addr, "58", Four);
        })
        .move_and_do(UpperRight, |&addr| {
            let _ = map.place_tile(addr, "9", One);
        })
        .move_and_do(UpperRight, |&addr| {
            let _ = map.place_tile(addr, "9", One);
        })
        .move_and_do(UpperRight, |&addr| {
            let _ = map.place_tile(addr, "9", One);
        });
}

/// Upgrades Moscow and Nizhnii Novgorod to green tiles.
fn place_green_tiles(map: &mut Map) {
    use RotateCW::*;
    moscow().do_here(|&addr| {
        let _ = map.place_tile(addr, "637", Zero);
    });
    nizhnii_novgorod().do_here(|&addr| {
        let _ = map.place_tile(addr, "207", Four);
    });
}

/// Upgrades Moscow and St Petersburg to brown tiles.
fn place_brown_tiles(map: &mut Map) {
    use RotateCW::*;
    moscow().do_here(|&addr| {
        let _ = map.place_tile(addr, "638", Zero);
    });
    st_petersburg().do_here(|&addr| {
        let _ = map.place_tile(addr, "641", Zero);
    });
}

/// Upgrades Moscow and Nizhnii Novgorod to grey tiles.
fn place_grey_tiles(map: &mut Map) {
    use RotateCW::*;
    moscow().do_here(|&addr| {
        let _ = map.place_tile(addr, "639", Zero);
    });
    st_petersburg().do_here(|&addr| {
        let _ = map.place_tile(addr, "642", Zero);
    });
}

/// Adds a bypass around Nizhnii Novgorod so that Moscow is directly connected
/// to Ekaterinburg.
fn place_skip_nizhnii_tiles(map: &mut Map) {
    use HexFace::*;
    use RotateCW::*;

    nizhnii_novgorod()
        .move_and_do(LowerLeft, |&addr| {
            let _ = map.place_tile(addr, "24", Two);
        })
        .move_and_do(LowerRight, |&addr| {
            let _ = map.place_tile(addr, "8", Four);
        })
        .move_and_do(UpperRight, |&addr| {
            let _ = map.place_tile(addr, "24", Five);
        });
}
