## Test cases

- `tests/connection_bonus`: also try requiring only one of the skipped dits, adding to_any options that are/are not on the path, including Toronto and Montreal, and so on.

- `tests/track`:

  - Test connections for some of the more complex tiles.

  - For each track segment, city, etc, we know the correct number of connections and can check that all of the expected connections are present.
    But this should go into a separate test file, `tile.rs` or `catalogue.rs`, and that file should also test drawing individual layers.
