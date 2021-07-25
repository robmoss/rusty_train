## UI state keymaps

Have each state return a `HashMap` or `BTreeMap` that maps specific mouse and keyboard events to handler objects that include a short name, a description, and a function that updates the current state or returns a new state.

```rust
pub enum UiEvent = { ... }
pub struct EventHandler {
    name: String,
    description: String,
    handler: Fn(UiEvent) -> UiEventResponse,
}
```

This would allow us to generate an automated description of the active key bindings and provide live help documentation in the UI.

But what type must the handler function have, and how do we ensure it only ever receives a (mutable) state value of the correct state type?
