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
