## n18tile

- `n18tile::city`: rename `Tokens` to `TokenSpaces` or something similar.

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
