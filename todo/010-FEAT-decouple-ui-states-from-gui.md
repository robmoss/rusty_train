## Decouple UI states from GUI

The intent is to support generating screenshots of the different UI states without having to launch a GTK application.

Perhaps the simplest way to achieve this is to have each separate event-handler response implemented as a separate method, which would allow us to generate screenshots along the lines of:

```rust
let surface = cairo::ImageSurface(...);
let ctx = cairo::Context::new(&surface)?;
let mut state = EditTokens::blah(...);
let content = new_dummy_content(...);
state.select_next_token_space();
state.select_next_token();
state.draw(&content, &ctx);
save_png(surface, ...);
```
