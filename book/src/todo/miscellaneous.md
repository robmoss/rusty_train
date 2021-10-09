# Miscellaneous items

This page collects smaller to-do items for each crate, the test cases, and the examples.

## n18io

Nothing.

## n18map

- `n18map::descr::update_map`: return a `Result`.

- `n18map::descr` 117-119: want `Map` to support placing tokens by name, similar to placing tiles?

- `n18map::map`: `ParseHexAddressError`, indicate in the returned error when we find an odd/even value instead of a required even/odd value.

- We may want to allow `Game` objects to use `Map::replace_tile` to replace tiles that are not otherwise replaceable.

- Modify `Map::prev_col`, `Map::next_col`, `Map::prev_row`, and `Map::next_row` to either

  - Return `Option<HexAddress>` values and return `None` if the previous/next address isn't a valid hex; or

  - Keep decreasing/increasing the column/row number until a valid address is found.

- Make `HexAddress` support more than 26 columns when converting to/from string coordinates.

## n18route

- `n18route::bonus`: to support the bonus off-board revenue for diesel trains in 1889, we need to add a `VisitWithTrainBonus`.
  This would only yield bonus revenue for the relevant off-board tiles if the route is operated by a **specific train type** (i.e., the diesel trains).

- `n18route::builder`: note in the doc strings that the "to_" prefix **is not** a type conversion; these connectivity functions.

- `n18route::comb`: odd that splitting at not-half-way gives worse performance:

  ```rust
  // let range = (self.ix0_max - self.current_ix) as f64;
  // let denom = 2.0_f64.powf(1.0 / self.max_len as f64);
  // let delta = (range / denom).round() as usize;
  // let split_at = self.current_ix + delta;
  ```

- `n18route::path::Path`:

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

- `n18tile::city`: `City::translate_coords()` uses custom adjustments for `HexPosition::Face` and `HexPosition::Corner`, and `City::delta_coords()` duplicates some of `HexPosition::to_coord()`.
  - Remove the custom position adjustments?
  - Define `Delta::coord(hex: &Hex, from: Coord) -> Coord` and use this in `HexPosition::to_coord()` and `City::translate_coords()`?

- Replace the `bool` field in `n18tile::label::PhaseRevenue` and `n18tile::label::PhaseRevenueVert` with a new enum type that has variants `Normal` and `Emphasise`?

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

- Add a new state that draws all of the track segments, etc, on off-board tiles, rather than only drawing the track segments on the off-board tile faces.
  Rather than adding a new flag to `Tile`, add a new `Tile` method that draws the tile and ignores the off-board special case, and add a new `n18brush::draw_tiles()` equivalent that calls this `Tile` method.

- Do not allow the user to place tokens on off-board tiles that have hidden revenue centres (i.e., tiles for which `tile.offboard_faces().is_some()` is `true`).

## Test cases

- `tests/connection_bonus`: also try requiring only one of the skipped dits, adding to_any options that are/are not on the path, including Toronto and Montreal, and so on.

- `n18catalogue`: test tile connections for most (all?) predefined tiles.

- `n18hex`, `n18tile`, `n18map`: write tests cases for coordinates, tile layout, and map connectivity for `Orientation::PointedTop`.

## n18game

Learn from the experience of implementing 1861 and 1867 and provide a variety of helper methods for implementing other games.

Consider dividing `n18game` into sub-modules:

- `tiles` (catalogue)
  - Provide a TileBuilder type
    - `.track(&mut self, Track)`
    - `.tracks(&mut self, IntoIterator<Item=Track>)`
    - `.city(&mut self, City)`
    - `.cities(&mut self, IntoIterator<Item=City>)`
    - `.onboard_faces(&mut self, IntoIterator<Item=HexFace>)`
    - `.build(&hex, colour, name: IntoString)`
  - Collect key game information in a single place
    - i.e., special tiles AND their locations / initial_state.

- `addrs` (define hex addresses and constants for each city)
  - Make each town and city's location a `static const` value?
  - Simplify defining the full range of map hexes
    - Allow `[A-Z]+[0-9]+` but must also support negative rows and columns.

- `map` (initial state, phases)

- `tokens` and/or `company`
  - May want to have tokens that are not part of a company for, e.g., national railways.

## n18catalogue

- Should `Kind::build()` also take a `n18hex::Orientation` argument, so that the positioning of tile elements (such as labels) can depend on the hex orientation?
