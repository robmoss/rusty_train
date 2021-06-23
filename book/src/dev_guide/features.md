# Features

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

Note that the `--manifest-path` arguments [are](https://github.com/rust-lang/cargo/issues/4753) [necessary](https://github.com/rust-lang/cargo/issues/5015) with Cargo's original [feature resolver](https://doc.rust-lang.org/cargo/reference/resolver.html).

## Updated feature resolver

As of [Rust 1.51](https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html) we have the option of enabling the "version 2" feature resolver, and avoiding the need for the `--manifest-path` arguments, by adding the following to the top-level `Cargo.toml`:

```toml
[package]
resolve = 2
```

This [changes the behavior](https://doc.rust-lang.org/cargo/reference/features.html) of the `--features` and `--no-default-features` command-line options, so that they enable/disable features for all workspace members.
Note that this resolver may also result in duplicated dependencies, which can be detected by running `cargo tree --duplicates`.
