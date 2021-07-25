# Implementation details

This page collects implementation details that should be added, changed, removed, or fixed.

## Invalid route-finding options

The route-finding algorithm assumes that routes can be constructed from one or more paths, where each path starts at a token T1 and never proceeds past a token T2 where T2 > T1, and we join pairs of paths that both start from the same token T and have no conflicts.

The path-building algorithm and route-finding algorithm both assume that a single path cannot pass through the same revenue centre more than once.
Note that the path-building algorithm stops whenever it encounters any kind of connection (hex face, track segment, revenue centre) that it has already visited.

The search criteria (`n18route::search::Criteria`) cannot allow `conflict_rule` to be `ConflictRule::TrackOnly` (i.e., no track segment in common), because this would mean that a single route could visit the same revenue centre multiple times, if there are sufficient track connections.

- So the implementation should panic, or return an `Error` value.

- Are there any games where this is a relevant concern?
  Note that this does not apply to "Flood" trains, which earn revenue from every revenue centre that can be reached from a single token (i.e., only requires a search from each matching token, and selecting the token that earns the most revenue).

See the `n18route::search` and `n18route::train` modules for the implementation.

## Game-specific route finding

The route-finding step that goes from `(trains, token, bonuses)` to `Routes` should be a method of the `n18game::Game` trait, because games may need to combine multiple search strategies if they include a variety of train types (hex trains, flood trains, etc).

- The `Game` trait can offer a default implementation.

- This method should probably accept `(&str, usize)` tuples, rather than accepting arbitrary `Train` values (which cannot be guaranteed to match any of the game's train types).
  Alternatively, provide this information as a `BTreeMap<&str, usize>` to ensure that there are no duplicate trains.

## Catalogue tiles by name

Tiles should be stored in a `HashMap` or `BTreeMap`, using the tile names as keys, so that games can assemble many of their tiles by simply providing a list of tile names.

- Provide this through a new `Catalogue` type that maps tile names to a `Tile` and an (optional) tile limit.

## Connectivity and themes

When a `Tile` is created, it determines the connections between all of the tile elements (tile edges, track segments, revenue centres) by identifying where these elements meet.
This, in turn, depends upon the theme settings associated with the `Hex` provided to `Tile::new()`.

- It may be preferable to use a specific theme to determine the tile connections, so that changes to a (user-facing) theme do not affect the structure of the track network.

- Change `n18catalogue` to create each `Tile` using the default `Hex` (including a **default hex diameter** of `125.0`).

## Error handling

The current implementation generally avoids returning `Result<T,E>` values and instead panics when an error is encountered.
Many of these panics are triggered by Cairo errors, such as failing to create a surface or context, or failing to draw on a surface, for which there is no obvious mitigation and panicking is an acceptable solution.

- But other panics should be removed and `Result<T,E>` values should be returned in their place.

Potential panics can be located with the following command:

```sh
grep --color=always -E '(\.unwrap|\.expect|panic!\()' -r crates/ tests/ examples/ src/
```

See [this article about error handling in Rust](https://www.lpalmieri.com/posts/error-handling-rust/), which frames error handling in terms of their **purpose** and **location**.

Relevant crates include [anyhow](https://github.com/dtolnay/anyhow), [eyre](https://github.com/yaahc/eyre), and [thiserror](https://github.com/dtolnay/thiserror).

## Builder patterns

Some of the more complex data structures would benefit from a **builder** to simplify their construction.
The preferred option is a [non-consuming builder](https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder) whose methods accept `&mut self` values and return `&mut Self` values.

I have implemented builders for some types (some consuming, some non-consuming), and have defined builder-like methods for other types (e.g., `n18hex::theme::Text`).

- There are likely other types for which a builder would be useful.

- Consuming builders should probably be converted into non-consuming builders.

## Remove n18io crate

It is possible to use [features to enable/disable (de)serialisation](https://rust-lang.github.io/api-guidelines/interoperability.html#data-structures-implement-serdes-serialize-deserialize-c-serde), which would remove the need for the `n18io` crate.

To include/exclude both serde and serde_json we need to define a single feature that enables/disables both crates:

```toml
[features]
load_save = ["serde", "serde_json"]
```

Note that optional dependencies [implicitly define](https://doc.rust-lang.org/cargo/reference/features.html) a feature of the same name as the dependency, and so explicit features cannot use the same name as a dependency.

But note that features and workspaces **are not easily combined**; see these issues — [1](https://github.com/rust-lang/cargo/issues/4463), [2](https://github.com/rust-lang/cargo/issues/5015), [3](https://github.com/rust-lang/cargo/issues/5251), [4](https://github.com/rust-lang/cargo/issues/9094) — for some perspective.

- It would appear that each crate would need to define this `load_save` feature, and for the `navig18xx` crate this feature would enable `crate-name/load_save` for each of these crates.

- The `navig18xx` crate should have no features enabled by default (i.e., not `load_save` or `ui`).
  These features can be enabled in the root `Cargo.toml` file so that they're available to the `rusty_train` binary.

## Use a single index for token spaces

Token spaces are currently indexed by revenue centre and by the token space number in that revenue centre.
Using a flat index `0..N` instead (or in addition) could make other parts of the code simpler and easier to understand.
For example, this would make it much simpler to show all placed tokens on replacement tiles.

## Separate combinations and permutations crates

- Consider splitting out the `n18route::comb` and `n18route::perm` modules into separate crates (e.g., `n18comb` and `n18perm`).

## Off-board locations

- Make being an off-board tile a property of the tile itself, rather than being determined purely by the tile's background colour.

- Assuming that these tiles are therefore route terminators, should we also draw track segments on these tiles as tapering to a point?
