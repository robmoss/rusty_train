## n18ui

- Add a new state that draws all of the track segments, etc, on off-board tiles, rather than only drawing the track segments on the off-board tile faces.
  Rather than adding a new flag to `Tile`, add a new `Tile` method that draws the tile and ignores the off-board special case, and add a new `n18brush::draw_tiles()` equivalent that calls this `Tile` method.
