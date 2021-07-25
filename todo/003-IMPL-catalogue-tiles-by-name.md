## Catalogue tiles by name

Tiles should be stored in a `HashMap` or `BTreeMap`, using the tile names as keys, so that games can assemble many of their tiles by simply providing a list of tile names.

- Provide this through a new `Catalogue` type that maps tile names to a `Tile` and an (optional) tile limit.
