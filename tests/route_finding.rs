use cairo::{Context, Format, ImageSurface};
use navig18xx::brush;
use navig18xx::prelude::*;
use std::collections::BTreeMap;

fn new_context(width: i32, height: i32) -> (Context, ImageSurface) {
    let surface = ImageSurface::create(Format::ARgb32, width, height)
        .expect("Can't create surface");
    let context =
        Context::new(&surface).expect("Can't create cairo::Context");
    (context, surface)
}

#[test]
/// This test checks whether two optimal routes that pass through different
/// token spaces on the same tile are correctly identified. This behaviour was
/// fixed by commit 6635eae.
fn test_dual_routes_from_montreal() {
    // Specify where to save the output images.
    let output_dir = std::path::Path::new("./tests/output");

    let hex_width: i32 = 125;
    let margin = 20;
    let columns = 16;
    let rows = 10;
    let (ctx, surf) = new_context(
        margin + columns * (hex_width as f32 * 0.78) as i32,
        margin + rows * (hex_width as f32 * 0.88) as i32,
    );
    let game = navig18xx::game::new_1867();
    let hex = Hex::new(hex_width as f64);
    let mut map = game.create_map(&hex);
    let company_token = *game.first_token();

    // Upgrade the Montreal tile and place two tokens.
    let addr_montreal = "L12".parse().unwrap();
    assert!(map.place_tile(addr_montreal, "X5", RotateCW::Zero));
    let hex_tile = map.tile_at(addr_montreal).unwrap();
    let space_0 = hex_tile.token_spaces()[0];
    let space_1 = hex_tile.token_spaces()[1];
    let map_hex = map.hex_mut(addr_montreal).unwrap();
    map_hex.set_token_at(&space_0, company_token);
    map_hex.set_token_at(&space_1, company_token);
    // Place the other tiles for these two routes.
    assert!(map.place_tile("M13".parse().unwrap(), "4", RotateCW::Zero));
    assert!(map.place_tile("M11".parse().unwrap(), "7", RotateCW::One));
    assert!(map.place_tile("M9".parse().unwrap(), "57", RotateCW::One));
    assert!(map.place_tile("L10".parse().unwrap(), "58", RotateCW::Zero));
    assert!(map.place_tile("K13".parse().unwrap(), "3", RotateCW::Four));

    let mut hex_iter = map.hex_iter(&hex, &ctx);
    brush::clear_surface(&ctx, Colour::WHITE);
    brush::draw_map(&hex, &ctx, &mut hex_iter);

    let filename = output_dir.join("test-dual-routes-montreal-map.png");
    let mut file = std::fs::File::create(filename)
        .expect("Couldn't create output PNG file");
    surf.write_to_png(&mut file)
        .expect("Couldn't write to output PNG file");

    // Search for the optimal routes.
    let all_trains: BTreeMap<&str, Train> = game
        .train_types()
        .into_iter()
        .map(|t| (game.train_name(t).unwrap(), *t))
        .collect();
    let company_trains: Trains =
        vec![*all_trains.get("4").unwrap(), *all_trains.get("4").unwrap()]
            .into();
    let limit = company_trains.path_limit();
    let criteria = Criteria {
        token: company_token,
        path_limit: limit,
        conflict_rule: ConflictRule::TrackOrCityHex,
        route_conflict_rule: ConflictRule::TrackOnly,
    };

    let paths = paths_for_token(&map, &criteria);
    let no_bonus = vec![];
    let best_opt = company_trains.select_routes(paths, no_bonus);
    assert!(best_opt.is_some());
    let best = best_opt.unwrap();
    assert!(best.train_routes.len() == 2);
    assert!(best.net_revenue == 230);

    // Draw each of the best routes, and save this to a PNG file.
    brush::clear_surface(&ctx, Colour::WHITE);
    brush::draw_map(&hex, &ctx, &mut hex_iter);
    brush::highlight_routes(&hex, &ctx, &map, &best.routes(), |ix| {
        hex.theme.nth_highlight_colour(ix)
    });
    let filename = output_dir.join("test-dual-routes-montreal-route.png");
    let mut file = std::fs::File::create(filename)
        .expect("Couldn't create output PNG file");
    surf.write_to_png(&mut file)
        .expect("Couldn't write to output PNG file");
}
