# Development to-do items

This page serves as a development road-map and bug tracker.

## Publishing to crates.io

See the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
for a lengthy checklist of guidelines that should be considered before
uploading this crate to the official [crate registry](https://crates.io/).

See notes about
[publishing workspaces](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-path-dependencies),
we need to specify both the ``path`` and the ``version``.

## Documentation

The documentation for each module needs to be fleshed out with
introductory text and examples, and more detailed documentation for
individual types and functions.
See the [Rust documentation guidelines](https://rust-lang.github.io/api-guidelines/documentation.html) for details.

## High-level features

### Avoid panics

The current implementation makes liberal use of ``.unwrap()``,
``.expect()``, and ``panic!()``. In many instances these could be replaced
by instead returning ``Result`` values and allowing errors to percolate up
the call stack.

All of these instances can be identified by running the following command:

```sh
grep --color=auto -E '(\.unwrap\(|\.expect\(|panic!\()' -r src crates
```

Consider using [thiserror](https://docs.rs/thiserror/*/thiserror/) within `navig18xx` and [anyhow](https://docs.rs/anyhow/*/anyhow/) in `rusty_train`, as per [this /r/rust error-handling discussion](https://redd.it/ej67aa)?

### Off-board locations

Make being an off-board tile a property of the tile itself, rather than being based on the tile's background colour?
Assuming that these tiles are therefore route terminators, should we also draw track segments on these tiles as tapering to a point?

### Company shares and dividends

Allow defining players and companies who may comprise some number of
shares, which can be owned by players, the company itself, etc.
Then the UI can automate the paying of dividends, allowing for things such
as full-pay, half-pay, and withhold.

This would go some way to providing features akin to
[18SH](https://github.com/msaari/18sh).
See the [BGG18SH thread](https://boardgamegeek.com/thread/2225619/18sh-command-line-replacement-spreadsheets) for ideas and planned features.
Complications include: rusting, nationalisation, mergers, loans, buying private companies and trains from other companies, etc.

+ Having found the optimal routes for a company, what about being able to press `d` to distribute full dividends, `h` to half-pay dividends, and `w` to withhold?

+ Having a game action log/console, into which players can enter commands and log games, undo actions, etc.
  For example, paying out dividends (or not) with any of the above commands from the UI could execute the appropriate command(s) in the console and log their output (e.g., pressing `d` could run a dividends command and also log a comment for each player who received money).

## Individual crates

### n18hex

Nothing.

### n18tile

Consider moving text-rendering code to a new crate, so that it can also be used by `n18tile` and `n18example`.

### n18token

Defer text positioning and drawing to `n18tile::Label`?

### n18catalogue

Nothing for now; ultimately, provide a broader range of 18xx tiles.

### n18map

Improve how tokens are managed, the current `Map::tokens()` method makes it quite fiddly to work with tokens.

### n18io

Nothing.

### n18route

Provide more flexibility in constructing `Train` values.

### n18game

Allow updated game phase by phase name, rather than by phase index.

### n18brush

Provide convenience methods that simplify the `State::draw()` methods for the UI states.

### n18ui

Only allow a tile to be upgraded when the replacement tile preserves the original tile's connections.

Record each action so that the full map history can be saved, and the user can walk backwards and forwards in time.

### n18example

Provide convenience methods to further simplify the examples in `./examples`.

### navig18xx

Nothing.
