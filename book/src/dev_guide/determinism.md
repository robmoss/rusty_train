# Determinism

By default, the Rust [HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html) and [HashSet](https://doc.rust-lang.org/std/collections/struct.HashMap.html) types use a randomly-seeded hashing algorithm, which means that they cannot be relied upon to provide a consistent ordering.

While it is possible to override the hashing algorithm, a simpler alternative is to use the [BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) and [BTreeSet](https://doc.rust-lang.org/std/collections/struct.BTreeSet.html) types, which require that the key type has a well-defined ordering (i.e, it must implement the [Ord](https://doc.rust-lang.org/std/cmp/trait.Ord.html) trait).

This has resulted in a small, but consistent, increase in [performance](./performance.md).
