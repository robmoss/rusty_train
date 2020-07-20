use cairo::{Context, Format, ImageSurface};
use n18brush;
use n18route::Bonus;
use navig18xx::prelude::*;
use std::collections::HashMap;

fn new_context(width: i32, height: i32) -> (Context, ImageSurface) {
    let surface = ImageSurface::create(Format::ARgb32, width, height)
        .expect("Can't create surface");
    (Context::new(&surface), surface)
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
    let hex_width: i32 = 125;
    let margin = 20;
    let columns = 16;
    let rows = 10;
    let (ctx, surf) = new_context(
        margin + columns * (hex_width as f32 * 0.78) as i32,
        margin + rows * (hex_width as f32 * 0.88) as i32,
    );
    let hex = Hex::new(hex_width as f64);
    let game = n18game::_1867::Game::new(&hex);
    let mut map = game.create_map(&hex);
    let company_token = game.company_tokens().first_token();

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
    assert!(map.place_tile("M13".parse().unwrap(), "3", RotateCW::Two));
    assert!(map.place_tile("M11".parse().unwrap(), "8", RotateCW::Two));
    assert!(map.place_tile("L10".parse().unwrap(), "58", RotateCW::One));
    assert!(map.place_tile("K11".parse().unwrap(), "8", RotateCW::Zero));
    assert!(map.place_tile("K13".parse().unwrap(), "58", RotateCW::Three));
    assert!(map.place_tile("J14".parse().unwrap(), "8", RotateCW::Four));
    assert!(map.place_tile("I13".parse().unwrap(), "8", RotateCW::One));
    assert!(map.place_tile("H14".parse().unwrap(), "3", RotateCW::Three));
    assert!(map.place_tile("H12".parse().unwrap(), "9", RotateCW::Zero));
    assert!(map.place_tile("H10".parse().unwrap(), "3", RotateCW::Zero));
    assert!(map.place_tile("G11".parse().unwrap(), "9", RotateCW::One));
    assert!(map.place_tile("F12".parse().unwrap(), "8", RotateCW::Zero));
    assert!(map.place_tile("F14".parse().unwrap(), "9", RotateCW::Zero));
    // Upgrade the Montreal tile.
    assert!(map.place_tile("L12".parse().unwrap(), "X2", RotateCW::Zero));
    // Upgrade the Toronto tile.
    assert!(map.place_tile("F16".parse().unwrap(), "120", RotateCW::Zero));

    // NOTE: must place an LP token for the train(s) to operate.
    let toronto = "F16".parse().unwrap();
    let hex_tile = map.tile_at(toronto).unwrap();
    let token_space = hex_tile.token_spaces()[1];
    let map_hex = map.get_hex_mut(toronto).unwrap();
    map_hex.set_token_at(&token_space, company_token);

    let mut hex_iter = map.hex_iter(&hex, &ctx);
    ctx.set_source_rgba(1.0, 1.0, 1.0, 1.0);
    ctx.paint();
    n18brush::draw_hex_backgrounds(&hex, &ctx, &mut hex_iter);
    n18brush::draw_tiles(&hex, &ctx, &mut hex_iter);
    n18brush::outline_empty_hexes(&hex, &ctx, &mut hex_iter);
    n18brush::draw_barriers(&hex, &ctx, &map);

    let filename = "test-conn-bonus-map.png";
    let mut file = std::fs::File::create(filename)
        .expect("Couldn't create output PNG file");
    surf.write_to_png(&mut file)
        .expect("Couldn't write to output PNG file");

    println!("Creating trains");
    let all_trains: HashMap<&str, Train> = game
        .train_types()
        .into_iter()
        .map(|t| (game.train_name(&t).unwrap(), t))
        .collect();
    let company_trains: Trains = vec![*all_trains.get("4").unwrap()].into();
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
    assert!(best.pairs.len() == 1);
    let best_path = &best.pairs[0].path;
    // NOTE: it picks L10 and M13 --- the two dits closest to Montreal.
    for visit in &best_path.visits {
        if visit.addr == "L10".parse().unwrap() {
            // Verify that this route stops at L10.
            assert!(visit.revenue > 0)
        } else if visit.addr == "M13".parse().unwrap() {
            // Verify that this route stops at M13.
            assert!(visit.revenue > 0)
        } else if visit.addr == "K13".parse().unwrap() {
            // Verify that this route skips K13.
            assert!(visit.revenue == 0)
        } else if visit.addr == "H10".parse().unwrap() {
            // Verify that this route skips H10.
            assert!(visit.revenue == 0)
        }
    }

    // Draw each of the best routes, and save this to a PNG file.
    ctx.set_source_rgba(1.0, 1.0, 1.0, 1.0);
    ctx.paint();
    n18brush::draw_hex_backgrounds(&hex, &ctx, &mut hex_iter);
    n18brush::draw_tiles(&hex, &ctx, &mut hex_iter);
    n18brush::outline_empty_hexes(&hex, &ctx, &mut hex_iter);
    n18brush::draw_barriers(&hex, &ctx, &map);
    n18brush::highlight_routes(&hex, &ctx, &map, &best.pairs, |ix| {
        match ix % 3 {
            0 => (0.7, 0.1, 0.1, 1.0),
            1 => (0.1, 0.7, 0.1, 1.0),
            _ => (0.1, 0.1, 0.7, 1.0),
        }
    });
    let filename = "test-conn-bonus-route-no-bonus.png";
    let mut file = std::fs::File::create(filename)
        .expect("Couldn't create output PNG file");
    surf.write_to_png(&mut file)
        .expect("Couldn't write to output PNG file");

    // Now add a connection bonus that requires the train to stop at two of
    // the skipped dits.
    let conn_bonus = vec![Bonus::ConnectionBonus {
        from: "K13".parse().unwrap(),
        to_any: vec!["H10".parse().unwrap()],
        bonus: 100,
    }];
    let new_best_conn_opt = company_trains.select_routes(paths, conn_bonus);
    assert!(new_best_conn_opt.is_some());
    let new_best = new_best_conn_opt.unwrap();
    assert!(new_best.net_revenue > best.net_revenue);
    assert!(new_best.net_revenue == 100 + best.net_revenue);
    assert!(new_best.pairs.len() == 1);
    let new_best_path = &new_best.pairs[0].path;
    for visit in &new_best_path.visits {
        if visit.addr == "L10".parse().unwrap() {
            // Verify that this route skips L10.
            assert!(visit.revenue == 0)
        } else if visit.addr == "M13".parse().unwrap() {
            // Verify that this route skips M13.
            assert!(visit.revenue == 0)
        } else if visit.addr == "K13".parse().unwrap() {
            // Verify that this route stops at K13.
            assert!(visit.revenue > 0)
        } else if visit.addr == "H10".parse().unwrap() {
            // Verify that this route stops at H10.
            assert!(visit.revenue > 0)
        }
    }

    // Draw each of the best routes, and save this to a PNG file.
    ctx.set_source_rgba(1.0, 1.0, 1.0, 1.0);
    ctx.paint();
    n18brush::draw_hex_backgrounds(&hex, &ctx, &mut hex_iter);
    n18brush::draw_tiles(&hex, &ctx, &mut hex_iter);
    n18brush::outline_empty_hexes(&hex, &ctx, &mut hex_iter);
    n18brush::draw_barriers(&hex, &ctx, &map);
    n18brush::highlight_routes(&hex, &ctx, &map, &new_best.pairs, |ix| {
        match ix % 3 {
            0 => (0.7, 0.1, 0.1, 1.0),
            1 => (0.1, 0.7, 0.1, 1.0),
            _ => (0.1, 0.1, 0.7, 1.0),
        }
    });
    let filename = "test-conn-bonus-route-with-bonus.png";
    let mut file = std::fs::File::create(filename)
        .expect("Couldn't create output PNG file");
    surf.write_to_png(&mut file)
        .expect("Couldn't write to output PNG file");
}
