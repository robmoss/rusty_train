## n18ui

- Make min/max hex size for zooming into constants / parameters?

- Pass `ctx` to `key_press_action` and `button_press_action` so that drawing can occur within each State's construction, if that construction involves long-running tasks (e.g., route-finding)?

- `FindRoutes`: fade out the entire map before starting the search?
  We would then need to redraw the previous state if `Find Routes::try_new()` returns `None`.

  ```rust
  ctx.set_source_rgba(1.0, 1.0, 1.0, 0.5);
  ctx.paint()
  some_widget.queue_redraw();
  ```

- May want to disable some of the global keybindings before the first game is created.
