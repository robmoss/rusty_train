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
