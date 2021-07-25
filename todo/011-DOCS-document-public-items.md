## Document public items

Identify public items that are missing documentation by running:

```sh
cargo clippy --all-targets -- -W missing_docs
```

For reference, see [this list of allowed-by-default lints](https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html).
