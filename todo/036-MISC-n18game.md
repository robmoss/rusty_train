## n18game

Learn from the experience of implementing 1861 and 1867 and provide a variety of helper methods for implementing other games.

Consider dividing `n18game` into sub-modules:

- `tiles` (catalogue)
  - Provide a TileBuilder type
    - `.track(&mut self, Track)`
    - `.tracks(&mut self, IntoIterator<Item=Track>)`
    - `.city(&mut self, City)`
    - `.cities(&mut self, IntoIterator<Item=City>)`
    - `.onboard_faces(&mut self, IntoIterator<Item=HexFace>)`
    - `.build(&hex, colour, name: IntoString)`
  - Collect key game information in a single place
    - i.e., special tiles AND their locations / initial_state.

- `addrs` (define hex addresses and constants for each city)
  - Make each town and city's location a `static const` value?
  - Simplify defining the full range of map hexes
    - Allow `[A-Z]+[0-9]+` but must also support negative rows and columns.

- `map` (initial state, phases)

- `tokens` and/or `company`
  - May want to have tokens that are not part of a company for, e.g., national railways.
