# Rust and Cargo features

This page lists items related to Rust language features and Cargo workflows.
It also collects feature requests and issues that would assist with this project.

## Publishing to crates.io

See the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) for a lengthy checklist of guidelines that should be considered before uploading this crate to the official [crate registry](https://crates.io/).

We also need to specify both the ``path`` and the ``version`` for each crate in the workspace, because [crates.io](https://crates.io/) [ignores path dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-path-dependencies).

The `cargo deb` [helper command](https://github.com/mmstick/cargo-deb) automatically creates binary Debian packages from Cargo projects, and [handles workspaces](https://github.com/mmstick/cargo-deb/issues/49).

## Rust version-related issues

See the "Minimum Supported Rust Version" [tracking issue](https://github.com/rust-lang/rust/issues/65262).

- [Rust 1.38](https://blog.rust-lang.org/2019/09/26/Rust-1.38.0.html) provides the [duration_float](https://blog.rust-lang.org/2019/09/26/Rust-1.38.0.html) feature.

- [Rust 1.48](https://blog.rust-lang.org/2020/11/19/Rust-1.48.html) provides [intra-doc links](https://doc.rust-lang.org/stable/rustdoc/linking-to-items-by-name.html).

- [Rust 1.51](https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html) allows [running all test cases](https://github.com/rust-lang/rust/pull/80053), and provides the "version 2" [feature resolver](https://doc.rust-lang.org/cargo/reference/features.html) that allows enabling/disabling [features](../dev_guide/features.md#updated-feature-resolver) for all crates in the workspace.

- [Rust 1.53](https://blog.rust-lang.org/2021/06/17/Rust-1.53.0.html) provides the [or_patterns](https://github.com/rust-lang/rfcs/pull/2535) feature, and is currently the minimum supported Rust version for the core `gtk-rs` crates.

## Workspace issues

- De-duplicate Cargo workspace information: [tracking issue](https://github.com/rust-lang/cargo/issues/8415).

## Documentation issues

- Include crate examples in the generated documentation:
[issue](https://github.com/rust-lang/cargo/issues/2760)

- Include images in the generated documentation: [issue](https://github.com/rust-lang/rust/issues/32104)

- Define enabled and disabled lints in a configuration file: [issue](https://github.com/rust-lang/cargo/issues/5034)

- Enable warnings for doctests: [relevant](https://github.com/rust-lang/rust/issues/41574) [issues](https://github.com/rust-lang/rust/issues/56232) and [pull request](https://github.com/rust-lang/rust/pull/73314).

## Testing issues

- Run all crate examples with a single command: [issue](https://github.com/rust-lang/cargo/issues/8356)

- Support `--nocapture` for doc tests: [issue](https://github.com/rust-lang/cargo/issues/1732)

- Running `cargo test --all-targets` does not run doc tests: [issue](https://github.com/rust-lang/cargo/issues/6669)
