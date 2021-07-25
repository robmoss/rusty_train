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
