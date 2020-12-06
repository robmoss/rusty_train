# Crates

The ``navig18xx`` crate is a wrapper that groups together a number of sub-crates:

- ``n18hex`` defines the basic geometry of hexagonal tiles (coordinates, faces, corners, background colours).
- ``n18tile`` defines the various elements that can appear on a tile (track segments, revenue centres, token spaces, labels) and constructs the track network for each tile.
- ``n18token`` defines the token types and manages the collection of available tokens
- ``n18catalogue`` defines the range of available tiles, including both the tiles that a player may place during a game, but also the tiles that make up the initial map (such as cities and towns, preexisting track segments, and off-board locations).
- ``n18map`` manages the state of a game map, such as tile and token placement.
- ``n18io`` defines ``Serialize`` and ``Deserialize`` implementations for the types defined by each of the above crates.
- ``n18route`` finds the optimal set of routes that can be operated by a company's trains for a given map state.
- ``n18game`` defines the elements that are required to describe a specific 18xx game implementation, and currently provides an (incomplete) implementation of 1867.
- ``n18brush`` defines common drawing operations, such as drawing the map background, drawing each map hex, and highlighting train routes.
- ``n18ui`` defines a GTK user interface for creating and modifying 18xx map states, and calculating the optimal revenue for each company.

The ``navig18xx`` crate exports the main public types, traits, values, and functions from these crates in the ``navig18xx::prelude`` module.

It also exports each of these crates without the ``n18`` prefix.
For example, ``n18hex`` is re-exported as ``navig18xx::hex``.

## Features

The `navig18xx` crate has one default feature: `ui`.
Disabling this feature removes the dependency on `n18ui` and GTK.
You can compile `navig18xx` without this feature with the following command:

```shell
cargo build --manifest-path crates/navig18xx/Cargo.toml -p navig18xx --no-default-features
```

Similarly, you can build the `navig18xx` documentation without this feature with the following command:

```shell
cargo doc --manifest-path crates/navig18xx/Cargo.toml -p navig18xx --no-default-features
```

Note that the `--manifest-path` arguments [are](https://github.com/rust-lang/cargo/issues/4753) [necessary](https://github.com/rust-lang/cargo/issues/5015) for now.
