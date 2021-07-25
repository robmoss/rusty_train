## Connectivity and themes

When a `Tile` is created, it determines the connections between all of the tile elements (tile edges, track segments, revenue centres) by identifying where these elements meet.
This, in turn, depends upon the theme settings associated with the `Hex` provided to `Tile::new()`.

- It may be preferable to use a specific theme to determine the tile connections, so that changes to a (user-facing) theme do not affect the structure of the track network.

- Change `n18catalogue` to create each `Tile` using the default `Hex` (including a **default hex diameter** of `125.0`).
