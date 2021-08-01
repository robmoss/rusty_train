## Catalogue tiles by name

Tiles should be stored in a `HashMap` or `BTreeMap`, using the tile names as keys, so that games can assemble many of their tiles by simply providing a list of tile names.

- Provide this through a new `Catalogue` type that maps tile names to a `Tile` and an (optional) tile limit.

- This will require cloning each `tile.name`, since we cannot use `&tile.name` as a key.

  - We could theoretically implement `Borrow<str>` for `Tile`, similar to the `CaseInsensitiveString` [example](https://doc.rust-lang.org/std/borrow/trait.Borrow.html), but this requires `Tile` and `str` to have identical `Hash` and `Eq` implementations, which isn't sensible.
