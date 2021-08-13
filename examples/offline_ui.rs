use navig18xx::prelude::*;
use navig18xx::ui::*;

mod output;
use output::Dir;

#[test]
fn test() {
    main()
}

/// Creates a new game of 1861, modifies the game state by feeding events to
/// the user interface, checks that these events have had the intended
/// effects, and saves a screenshot.
fn main() {
    let image_dir = Dir::Examples;
    let output_file = image_dir.join("offline_ui.png");
    let ui = build_ui();
    check_ui(&ui);
    save_screenshot(&ui, output_file);
}

/// Creates a new game of 1861, places a token on the Moscow tile, rotates the
/// Moscow tile, and upgrades the Moscow tile from yellow to green.
fn build_ui() -> UserInterface {
    let games: Vec<Box<dyn Game>> = vec![
        Box::new(navig18xx::game::new_1861()),
        Box::new(navig18xx::game::new_1867()),
    ];
    let controller: Controller = control::DummyController::new().into();
    let keymap = Keymap::default();
    let mut ui = UserInterface::new(games, controller, keymap);

    // Create a new game of 1861.
    let response = ui.new_game(0);
    ui.respond(response);

    // Make Moscow the active hex for the default UI state.
    // NOTE: alternatively, feed 4x Down and 8x Left key events, since the
    // initial active hex is the top-right grey tile.
    let moscow: HexAddress = (4, 7).into();
    ui.state = state::default::Default::at_hex(moscow).into();
    ui.draw();

    // Place a token in the bottom token space.
    feed_key(&mut ui, gdk::keys::constants::t);
    feed_key(&mut ui, gdk::keys::constants::Up);
    feed_key(&mut ui, gdk::keys::constants::Up);
    feed_key(&mut ui, gdk::keys::constants::Return);

    // Rotate the tile two turns clockwise.
    feed_key(&mut ui, gdk::keys::constants::period);
    feed_key(&mut ui, gdk::keys::constants::period);

    // Upgrade the Moscow tile to green.
    feed_key(&mut ui, gdk::keys::constants::u);
    feed_key(&mut ui, gdk::keys::constants::Return);

    ui
}

/// Feeds a key-press event to the user interface.
fn feed_key(ui: &mut UserInterface, key: keymap::Key) {
    let (ctrl, alt, shift) = (false, false, false);
    let event = KeyPress {
        key,
        ctrl,
        alt,
        shift,
    };
    let response = ui.handle_key_press(&event);
    ui.respond(response);
}

/// Tests whether the UI actions have had the intended effects.
fn check_ui(ui: &UserInterface) {
    let moscow: HexAddress = (4, 7).into();
    let hex_state_opt = ui.assets.map.hex_state(moscow);
    // Ensure that there is a tile placed on the Moscow hex.
    assert!(hex_state_opt.is_some());
    let hex_state = hex_state_opt.unwrap();
    // Check the placed tile's name and rotation are as expected.
    assert_eq!(hex_state.tile(&ui.assets.map).name, "637");
    assert_eq!(hex_state.rotation(), &RotateCW::Two);
    // Check that there is a token placed in the first token space.
    let tok_spaces = hex_state.tile(&ui.assets.map).token_spaces();
    let token_opt = hex_state.token_at(&tok_spaces[0]);
    assert!(token_opt.is_some());
    // Check that the placed token has the expected name.
    let token = token_opt.unwrap();
    assert_eq!(ui.assets.map.try_token_name(token), Some("KK"));
}

/// Saves a screenshot of the current UI state.
fn save_screenshot(ui: &UserInterface, dest: std::path::PathBuf) {
    let margin = 1;
    let image = ui
        .canvas
        .copy_ink_with_margin(&ui.state, &ui.assets, margin);
    let _ = ui.save_image(dest, image);
}
