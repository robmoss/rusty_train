/// Find optimal routes for various trains in different phases of 1889, and
/// ensure that the off-board bonuses for Diesel trains are applied correctly.
use log::info;
use navig18xx::game::_1889;
use navig18xx::prelude::*;
use std::io::Write;

/// The different phases and trains for which to find the optimal routes, and
/// the expected optimal revenues.
const REVENUES: [(&str, &str, usize); 6] = [
    ("2", "2", 50),
    ("3", "3", 80),
    ("4", "4", 80),
    ("5", "5", 140),
    ("6", "6", 140),
    ("D", "D", 220),
];

/// Find the optimal routes for different trains and game phases.
#[test]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    for &(phase, train, revenue) in &REVENUES {
        find_optimal_route(phase, train, revenue)?;
    }
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

fn find_optimal_route(
    phase: &str,
    train: &str,
    expected: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // Specify where to save the output images.
    let output_dir = std::path::Path::new("./tests/output");

    let hex = Hex::default();
    let mut game = navig18xx::game::new_1889();
    let mut map = game.create_map(&hex);
    game.set_phase_name(&mut map, phase);

    // Place a token on the starting (yellow) Takamatsu tile.
    // It should remain in place with each tile upgrade.
    let token_addr = _1889::Location::Takamatsu.address();
    let token = map.token("AR");
    let token_space = map.tile_at(token_addr).unwrap().token_spaces()[0];
    let hex_state = map.hex_state_mut(token_addr).unwrap();
    hex_state.set_token_at(&token_space, token);

    // Place tiles as appropriate for the chosen game phase.
    let phase_num = 2 + game.phase_ix();
    let orient = game.hex_orientation();

    token_addr.move_and_do(HexFace::UpperLeft, orient, |&addr| {
        map.place_tile(addr, "8", RotateCW::Five);
    });
    token_addr.move_and_do(HexFace::Bottom, orient, |&addr| {
        map.place_tile(addr, "8", RotateCW::Five);
    });

    // Phase 3: upgrade Takamatsu to $40 with tile 440.
    if phase_num >= 3 {
        map.place_tile(token_addr, "440", RotateCW::Zero);
    }

    // Phase 5: upgrade Takamatsu to $60 with tile 466.
    if phase_num >= 5 {
        map.place_tile(token_addr, "466", RotateCW::Zero);
    }

    // Identify the optimal train route.
    let trains = Trains::new(vec![*game.train(train)]);
    let bonus_flags: Vec<bool> = vec![];
    let routes = game.best_routes(&map, token, &trains, bonus_flags).unwrap();
    info!(
        "Phase {} Train {} earned ${}, expected ${}",
        phase, train, routes.net_revenue, expected
    );
    assert_eq!(routes.net_revenue, expected);

    // Draw the map and highlight the optimal route.
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

    // Add a revenue label.
    let labeller = navig18xx::hex::theme::Text::new()
        .font_size(36.0)
        .halign_centre()
        .valign_bottom()
        .font_serif()
        .bold()
        .labeller(&rec_ctx, &hex);
    let addr = game.coordinate_system().parse("B1").unwrap();
    let m = map.prepare_to_draw(addr, &hex, &rec_ctx);
    labeller.draw(
        &format!("{}-train: ${}", train, routes.net_revenue),
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
    let basename = format!("1889_diesel_bonus_{phase}_{train}-train.png");
    let filename = output_dir.join(basename);
    println!("Writing {} ...", filename.to_str().unwrap());
    let mut file =
        std::fs::File::create(filename).expect("Can't create output file");
    surf.write_to_png(&mut file)
        .expect("Can't write output file");

    Ok(())
}
