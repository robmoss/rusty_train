# Miscellaneous items

This page collects smaller to-do items for each crate, the test cases, and the examples.

## n18io

- `n18io/src/lib.rs`:

  ```rust
  // TODO: what about to centre for City ... is it relevant as per Label?
  // NOTE: I think it's just as relevant, and only not an issue because there
  // may not be any city on any tile that has a ToCentre delta.
  pub nudge: Option<(Direction, f64)>
  ```

## n18map

- `n18map::descr::update_map`: return a `Result`.

- `n18map::descr` 117-119: want `Map` to support placing tokens by name, similar to placing tiles?

- `n18map`: instead of repeated `.clockwise()` and `.anti_clockwise()` turns, implement `std::ops::Add<usize>` and `std::ops::Sub<usize>` for `HexFace`?

- `n18map`: improve how tokens are managed, the current `Map::tokens()` method makes it quite fiddly to work with tokens.

- `n18map::Map::new()`
  Take `tokens: T` where `T: IntoIterator<Item = (S, Token)>` and `S: ToString`.

  - Note that this means `Map` can **own** the token names.

  - `n18map::descr::build_map`: change how tokens are passed to `Map::new()`, use `tokens.into_iter()`.

- `n18map::Map`: when updating a tile, do not "leave the tokens as-is (presumably this is the correct behaviour?)".
  Instead, only retain tokens associated with token spaces that are valid for the new tile.

- `n18map::Map::prepare_to_draw()`: document that this merely translates the origin, it does not define a current point on the current path, so you cannot use `ctx.rel_move_to()` without first defining a current point.

- `n18map::MapHex`: does this still need ergonomic improvements?

- `n18map::map`: `ParseHexAddressError`, indicate in the returned error when we find an odd/even value instead of a required even/odd value.

## n18route

- `n18route::builder`: note in the doc strings that the "to_" prefix **is not** a type conversion; these connectivity functions.

- `n18route::comb`: odd that splitting at not-half-way gives worse performance:

  ```rust
  // let range = (self.ix0_max - self.current_ix) as f64;
  // let denom = 2.0_f64.powf(1.0 / self.max_len as f64);
  // let delta = (range / denom).round() as usize;
  // let split_at = self.current_ix + delta;
  ```

- `n18route::path::Path`:

  - replace `HashSet` field with `BTreeSet` so that `Path` values can be hashed?
  - add an `append(other: Path)` method to `Path`?

- `n18route::path`: distinguish between `Path` (which defines track segments, hex faces, and cities that a train passes through) and `Route` (which defines the visits that the train makes).

- `n18route::search`:

  - use `rayon` to iterate over `connections.iter()` in parallel?

  - use `rayon` to iterate over `paths.iter().enumerate()` in parallel?

  - `depth_first_search()`: should we allow starting at a dit?
    If not, store the starting city_ix in the context?

  - Adjacent red hexes are considered the same location and cannot be visited multiple times, so we should probably have adjacent red tiles contain a single city with track that connects to all of the other adjacent red tiles ... or have `Tile` define this instead (e.g., `tile.is_offboard()`)?

    Further on, `let off_board = tile_colour = ...`, should `Tile` define this instead?

  - `n18route::search::tests`: also modify the map so that `paths_from` and `paths_through` return different revenues, and we can check that they're each correct.

## n18tile

- `n18tile::city`: rename `Tokens` to `TokenSpaces` or something similar.

- `n18tile::Label::Y`: this should store a String argument, so ... does this differ from `City()` then?
  Note that we allow map hexes and tiles to have multiple such labels.

- Allow labels to have a custom anchor/alignment?
  But where to store this?
  Or pass it as an optional argument to the drawing function (although this doesn't solve the where-to-store-this question)?
  Would be handy for having `MapLocation` labels with consistent vertical alignment with 1-line or 2-line text ... the alternative is to position them at `Centre` and nudge them up ... nudge some frac to `Face::Top`.

- `n18tile::tile::Tile`: indicate which tiles are available for players to place, as opposed to being tiles internal to the game map.
  But this is more of a per-game concern, and should be defined by each `n18game::Game` instance.
  So rather than being a (mutable) `Tile` property, the `Game` should return the collection of all Tiles (asset and player), and separately return the collection of all available-to-player tiles.

- `n18tile::tile::Tile`:
  - Break out the layer calculations into a separate struct, similar to `connection::Connections`?
  - Expose functions for drawing layers for integration tests?
    - `pub fn tracks_in_layer(&self, layer) -> ?Vec?`
    - `pub fn cities_in_layer(&self, layer) -> ?Vec?`
  - Mark track segments on red (and blue) tiles as terminal?
    - Involves adding a `pub terminal: bool` field to `Track`, with a default value of `false`, and adding a method `mark_as_terminal()`

- `n18tile::track::Track`: define a private `dit_direction(&self, hex: &Hex) -> Option<Coord>` method?

  - The `Track` type really needs an internal `dit_coord()` method, it would replace a lot of duplicated code.

  - Verify that `Track::dit_coord()` actually agrees with the dit location!

  - Make `track::Coords` use `Track::get_coord()` for iteration, so that there's only one piece of code that calculates track coordinates.

## n18token

- `n18token:Token`: it would be nice if each token owned its name, but then Token cannot implement Copy ...

  - This should be in the type and/or module documentation.

- `n18token:Tokens`: implement `IntoIterator` for `Item = (String, Token)` ... be we need to specify the exact `Iterator` type, so we'd have to make our own struct that implements `Iterator`.

## n18ui

- Make min/max hex size for zooming into constants / parameters?

- Pass `ctx` to `key_press_action` and `button_press_action` so that drawing can occur within each State's construction, if that construction involves long-running tasks (e.g., route-finding)?

- `FindRoutes`: fade out the entire map before starting the search?
  We would then need to redraw the previous state if `Find Routes::try_new()` returns `None`.

  ```rust
  ctx.set_source_rgba(1.0, 1.0, 1.0, 0.5);
  ctx.paint()
  some_widget.queue_redraw();
  ```

- May want to disable some of the global keybindings before the first game is created.

## Test cases

- `tests/connection_bonus`: also try requiring only one of the skipped dits, adding to_any options that are/are not on the path, including Toronto and Montreal, and so on.

- `tests/track`:

  - Test connections for some of the more complex tiles.

  - For each track segment, city, etc, we know the correct number of connections and can check that all of the expected connections are present.
    But this should go into a separate test file, `tile.rs` or `catalogue.rs`, and that file should also test drawing individual layers.
