use cairo::{Context, Format, ImageSurface};
use navig18xx::brush;
use navig18xx::prelude::*;
use navig18xx::route::Bonus;

fn new_context(width: i32, height: i32) -> (Context, ImageSurface) {
    let surface = ImageSurface::create(Format::ARgb32, width, height)
        .expect("Can't create surface");
    let context =
        Context::new(&surface).expect("Can't create cairo::Context");
    (context, surface)
}

#[test]
/// This test checks whether a connection bonus that is not satisfied when
/// stopping at the highest-revenue stops along a path is correctly identified
/// as the optimal route.
///
/// This is done by setting up a map so that there is a path between two
/// cities that passes through a number of dits with different revenues, in
/// combination with a very large bonus for connecting the two lowest-revenue
/// dits on the path. For a company that owns a single 4-train, the optimal
/// route should be to connect the two cities, and of the remaining two stops
/// that it can make, to satisfy the connection bonus rather than targeting
/// the intermediate locations with the highest base revenue.
fn test_connection_bonus_between_two_dits() {
    // Specify where to save the output images.
    let output_dir = std::path::Path::new("./tests/output");
    let book_dir = std::path::Path::new("./book/src");

    let hex_width: i32 = 125;
    let margin = 20;
    let columns = 16;
    let rows = 10;
    let (ctx, surf) = new_context(
        margin + columns * (hex_width as f32 * 0.78) as i32,
        margin + rows * (hex_width as f32 * 0.88) as i32,
    );
    let game = navig18xx::game::new_1867();
    let coords = game.coordinate_system();
    let hex = Hex::new(hex_width as f64);
    let mut map = game.create_map(&hex);
    let company_token = *game.first_token();

    // Define the map state. Note that all 1861/67 dit tiles earn 10 revenue.
    // We first calculate the optimal route in the absence of any connection
    // bonuses, and then define a connection bonus that involves two of the
    // skipped dits.
    //
    // From Montreal (L12) we need to add:
    // * M13 #3  CW2   dit
    // * M11 #8  CW2
    // * L10 #58 CW1   dit
    // * K11 #8  Zero
    // * K13 #58 180   dit
    // * J14 #8  ACW2
    // * I13 #8  CW1
    // * H14 #3  180   dit
    // * H12 #9  Zero
    // * H10 #3  Zero  dit
    // * G11 #9  CW1
    // * F12 #8  Zero
    // * F14 #9  Zero
    println!("Placing tiles");
    assert!(map.place_tile(coords.parse("M13").unwrap(), "3", RotateCW::Two));
    assert!(map.place_tile(coords.parse("M11").unwrap(), "8", RotateCW::Two));
    assert!(map.place_tile(
        coords.parse("L10").unwrap(),
        "58",
        RotateCW::One
    ));
    assert!(map.place_tile(
        coords.parse("K11").unwrap(),
        "8",
        RotateCW::Zero
    ));
    assert!(map.place_tile(
        coords.parse("K13").unwrap(),
        "58",
        RotateCW::Three
    ));
    assert!(map.place_tile(
        coords.parse("J14").unwrap(),
        "8",
        RotateCW::Four
    ));
    assert!(map.place_tile(coords.parse("I13").unwrap(), "8", RotateCW::One));
    assert!(map.place_tile(
        coords.parse("H14").unwrap(),
        "3",
        RotateCW::Three
    ));
    assert!(map.place_tile(
        coords.parse("H12").unwrap(),
        "9",
        RotateCW::Zero
    ));
    assert!(map.place_tile(
        coords.parse("H10").unwrap(),
        "3",
        RotateCW::Zero
    ));
    assert!(map.place_tile(coords.parse("G11").unwrap(), "9", RotateCW::One));
    assert!(map.place_tile(
        coords.parse("F12").unwrap(),
        "8",
        RotateCW::Zero
    ));
    assert!(map.place_tile(
        coords.parse("F14").unwrap(),
        "9",
        RotateCW::Zero
    ));
    // Upgrade the Montreal tile.
    assert!(map.place_tile(
        coords.parse("L12").unwrap(),
        "X2",
        RotateCW::Zero
    ));
    // Upgrade the Toronto tile.
    assert!(map.place_tile(
        coords.parse("F16").unwrap(),
        "120",
        RotateCW::Zero
    ));

    // NOTE: must place an LP token for the train(s) to operate.
    let toronto = coords.parse("F16").unwrap();
    let hex_tile = map.tile_at(toronto).unwrap();
    let token_space = hex_tile.token_spaces()[1];
    let map_hex = map.hex_state_mut(toronto).unwrap();
    map_hex.set_token_at(&token_space, company_token);

    let mut hex_iter = map.hex_iter(&hex, &ctx);
    brush::clear_surface(&ctx, Colour::WHITE);
    brush::draw_map(&hex, &ctx, &mut hex_iter);

    let filename = output_dir.join("test-conn-bonus-map.png");
    let mut file = std::fs::File::create(filename)
        .expect("Couldn't create output PNG file");
    surf.write_to_png(&mut file)
        .expect("Couldn't write to output PNG file");

    println!("Creating trains");
    let company_trains: Trains = vec![*game.train("4")].into();
    let limit = company_trains.path_limit();
    let criteria = Criteria {
        token: company_token,
        path_limit: limit,
        conflict_rule: ConflictRule::TrackOrCityHex,
        route_conflict_rule: ConflictRule::TrackOnly,
    };

    let paths = paths_for_token(&map, &criteria);
    let no_bonus = vec![];
    let best_opt = company_trains.select_routes(paths.clone(), no_bonus);
    assert!(best_opt.is_some());
    let best = best_opt.unwrap();
    assert!(best.train_routes.len() == 1);
    let best_route = &best.train_routes[0].route;
    // NOTE: it picks L10 and M13 --- the two dits closest to Montreal.
    let l_10 = coords.parse("L10").unwrap();
    let m_13 = coords.parse("M13").unwrap();
    let k_13 = coords.parse("K13").unwrap();
    let h_10 = coords.parse("H10").unwrap();
    for visit in &best_route.visits {
        if visit.addr == l_10 || visit.addr == m_13 {
            // Verify that this route stops at L10 and M13.
            assert!(visit.revenue > 0)
        } else if visit.addr == k_13 || visit.addr == h_10 {
            // Verify that this route skips K13 and H10.
            assert!(visit.revenue == 0)
        }
    }

    // Draw each of the best routes, and save this to a PNG file.
    brush::clear_surface(&ctx, Colour::WHITE);
    brush::draw_map(&hex, &ctx, &mut hex_iter);
    brush::highlight_routes(&hex, &ctx, &map, &best.routes(), |ix| {
        hex.theme.nth_highlight_colour(ix)
    });
    let filename = output_dir.join("test-conn-bonus-route-no-bonus.png");
    let mut file = std::fs::File::create(filename)
        .expect("Couldn't create output PNG file");
    surf.write_to_png(&mut file)
        .expect("Couldn't write to output PNG file");
    // Also save this image to the book directory.
    let filename = book_dir.join("test-conn-bonus-route-no-bonus.png");
    let mut file = std::fs::File::create(filename)
        .expect("Couldn't create output PNG file");
    surf.write_to_png(&mut file)
        .expect("Couldn't write to output PNG file");

    // Now add a connection bonus that requires the train to stop at two of
    // the skipped dits.
    let conn_bonus = vec![Bonus::ConnectionBonus {
        from: coords.parse("K13").unwrap(),
        to_any: vec![coords.parse("H10").unwrap()],
        bonus: 100,
    }];
    let new_best_conn_opt = company_trains.select_routes(paths, conn_bonus);
    assert!(new_best_conn_opt.is_some());
    let new_best = new_best_conn_opt.unwrap();
    assert!(new_best.net_revenue > best.net_revenue);
    assert!(new_best.net_revenue == 100 + best.net_revenue);
    assert!(new_best.train_routes.len() == 1);
    let new_best_path = &new_best.train_routes[0].route;
    for visit in &new_best_path.visits {
        if visit.addr == l_10 || visit.addr == m_13 {
            // Verify that this route skips L10 and M13.
            assert!(visit.revenue == 0)
        } else if visit.addr == k_13 || visit.addr == h_10 {
            // Verify that this route stops at K13 and H10.
            assert!(visit.revenue > 0)
        }
    }

    // Draw each of the best routes, and save this to a PNG file.
    brush::clear_surface(&ctx, Colour::WHITE);
    brush::draw_map(&hex, &ctx, &mut hex_iter);
    brush::highlight_routes(&hex, &ctx, &map, &new_best.routes(), |ix| {
        hex.theme.nth_highlight_colour(ix)
    });
    let filename = output_dir.join("test-conn-bonus-route-with-bonus.png");
    let mut file = std::fs::File::create(filename)
        .expect("Couldn't create output PNG file");
    surf.write_to_png(&mut file)
        .expect("Couldn't write to output PNG file");
}
