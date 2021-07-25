## n18io

- `n18io/src/lib.rs`:

  ```rust
  // TODO: what about to centre for City ... is it relevant as per Label?
  // NOTE: I think it's just as relevant, and only not an issue because there
  // may not be any city on any tile that has a ToCentre delta.
  pub nudge: Option<(Direction, f64)>
  ```
