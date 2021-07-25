## Publishing to crates.io

See the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) for a lengthy checklist of guidelines that should be considered before uploading this crate to the official [crate registry](https://crates.io/).

We also need to specify both the ``path`` and the ``version`` for each crate in the workspace, because [crates.io](https://crates.io/) [ignores path dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-path-dependencies).

The `cargo deb` [helper command](https://github.com/mmstick/cargo-deb) automatically creates binary Debian packages from Cargo projects, and [handles workspaces](https://github.com/mmstick/cargo-deb/issues/49).
