## n18map

- `n18map::descr::update_map`: return a `Result`.

- `n18map::descr` 117-119: want `Map` to support placing tokens by name, similar to placing tiles?

- `n18map::map`: `ParseHexAddressError`, indicate in the returned error when we find an odd/even value instead of a required even/odd value.

- We may want to allow `Game` objects to use `Map::replace_tile` to replace tiles that are not otherwise replaceable.

- Modify `Map::prev_col`, `Map::next_col`, `Map::prev_row`, and `Map::next_row` to either

  - Return `Option<HexAddress>` values and return `None` if the previous/next address isn't a valid hex; or

  - Keep decreasing/increasing the column/row number until a valid address is found.

- Make `HexAddress` support more than 26 columns when converting to/from string coordinates.
