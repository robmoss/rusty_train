# Rust and Cargo features

This page collects Rust and Cargo features and feature requests that would assist with this project.

- Provide "Minimum Supported Rust Version": [issue](https://github.com/rust-lang/rust/issues/65262)

  - [Rust 1.38](https://blog.rust-lang.org/2019/09/26/Rust-1.38.0.html) provides the [duration_float](https://blog.rust-lang.org/2019/09/26/Rust-1.38.0.html) feature.

  - [Rust 1.48](https://blog.rust-lang.org/2020/11/19/Rust-1.48.html) provides [intra-doc links](https://doc.rust-lang.org/stable/rustdoc/linking-to-items-by-name.html).

  - [Rust 1.51](https://blog.rust-lang.org/2021/03/25/Rust-1.51.0.html) allows [running all test cases](https://github.com/rust-lang/rust/pull/80053), and provides the "version 2" [feature resolver](https://doc.rust-lang.org/cargo/reference/features.html) that allows enabling/disabling [features](../dev_guide/features.md#updated-feature-resolver) for all crates in the workspace.

    This is the minimum supported Rust version for the `gtk-rs` version 0.14 crates.

    However, the build dependency `system-deps` 3.2.0 depends on `cfg-expr` 0.8.0, which requires the [or_patterns](https://github.com/rust-lang/rfcs/pull/2535) feature from Rust 1.53.
    This can be circumvented by setting the precise version of `system-deps`:

    ```sh
    cargo update -p system-deps --precise 3.1.2
    ```

- De-duplicate Cargo workspace information: [issue](https://github.com/rust-lang/cargo/issues/8415)

- Include crate examples in the generated documentation:
[issue](https://github.com/rust-lang/cargo/issues/2760)

- Run all crate examples with a single command: [issue](https://github.com/rust-lang/cargo/issues/8356)

- Support `--nocapture` for doc tests: [issue](https://github.com/rust-lang/cargo/issues/1732)
