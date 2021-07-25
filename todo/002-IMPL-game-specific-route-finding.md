## Game-specific route finding

The route-finding step that goes from `(trains, token, bonuses)` to `Routes` should be a method of the `n18game::Game` trait, because games may need to combine multiple search strategies if they include a variety of train types (hex trains, flood trains, etc).

- The `Game` trait can offer a default implementation.

- This method should probably accept `(&str, usize)` tuples, rather than accepting arbitrary `Train` values (which cannot be guaranteed to match any of the game's train types).
  Alternatively, provide this information as a `BTreeMap<&str, usize>` to ensure that there are no duplicate trains.
