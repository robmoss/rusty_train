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
