## Builder patterns

Some of the more complex data structures would benefit from a **builder** to simplify their construction.
The preferred option is a [non-consuming builder](https://rust-lang.github.io/api-guidelines/type-safety.html#builders-enable-construction-of-complex-values-c-builder) whose methods accept `&mut self` values and return `&mut Self` values.

I have implemented builders for some types (some consuming, some non-consuming), and have defined builder-like methods for other types (e.g., `n18hex::theme::Text`).

- There are likely other types for which a builder would be useful.

- Consuming builders should probably be converted into non-consuming builders.
