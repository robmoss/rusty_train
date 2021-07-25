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
