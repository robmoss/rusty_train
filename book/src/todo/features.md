# Planned features

## Valid tile upgrades

Across most (all?) 18xx games there are [three different rules for upgrading track](https://www.railsonboards.com/2020/12/26/permissive-restrictive-semi-restrictive-what-it-means/):

- Permissive;
- Semi-Restrictive; and
- Restrictive.

We could define an `UpgradeRule` enum with three variants.
This would allow us to:

1. Identify candidate upgrade tiles (noting that candidates may not be valid for all possible tile rotations);

2. Validate the chosen tile and rotation; and

3. Place each token from the original tile in an appropriate token space.

**NOTE:** the valid candidates may depend on which company is performing the upgrade, so candidate selection and validation will both require passing a valid `Token` to identify the company.

Are there any situations where the `Game` itself should have some say in choosing candidates and/or validating the chosen replacement?

- Given that some 18xx games include events that alter the map, such as the North-West Rebellion in 1882: Assiniboia, it is probably best to provide a default implementation (e.g., as a default method for `n18game::Game`) and allow individual games to override this as required.

- Or should we require each game to define the valid upgrade tiles, and only provide a default implementation for validation?
  This method could look something like:

  ```rust
  fn upgrades_for(&self, tile_name: &str) -> Option<Vec<&str>>
  ```

- Games can then hard-code the upgrade options, and we only need to implement the validation logic and token placement for each of the `UpgradeRule` variants.

### Representation

We would need to define a data structure that characterises the current tile's topology and connections with respect to the company performing the upgrade.

### UI states

The current `ReplaceTile` state should be separated into two states:

- A `ReplaceTile` state that only handles replacements, not upgrades (i.e., ignores placed tokens and upgrade concerns); and

- An `UpgradeTile` state that describes the current tile's properties (see above) and only accepts valid upgrades.
  Whenever the candidate tile is changed or rotated, the hex border colour could be used to indicate whether the current configuration is a valid upgrade.

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

## Undo/redo

Any UI event-handler that modifies the map should return an `Action` or `Command` enum that knows how to make **and** revert this modification to the map.
The UI can then maintain a vector of past actions and an index to the current undo position, allowing the user to undo and redo these actions.
Performing an action other than undo or redo would clear the future actions, and append this new action to the past actions.

The [Command pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/command.html) might be useful here.
Also see [these](https://redd.it/muei0l) [two](https://redd.it/mtknz0) `/r/rust` discussions about implementing undo/redo, and the [undo crate](https://github.com/evenorog/undo).

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

## Port to GTK 4

The [gtk-rs project](https://gtk-rs.org/) have released a [GTK 4 crate](https://crates.io/crates/gtk4) and an [introductory book](https://gtk-rs.org/gtk4-rs/stable/latest/book/).

It may be as simple as using the `gtk4` and `gdk4` crates, and making a few changes to the `rusty_train` binary.

Note that GTK 4 is not yet packages for Debian stable, testing, or unstable.
See [Debian bug 992907](https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=992907) for progress.

## 18xx-maker JSON schema

See the [18xx-maker repository](https://github.com/18xx-maker/18xx-maker/), specifically the `src/data/tiles` directory, for examples of tile definitions that are used to create game maps such as [1867](https://www.18xx-maker.com/games/1867/map).

Perhaps it would be possible to translate between the 18xx-maker data format and that of `n18io`.

## Company shares and dividends

Once the optimal routes have been identified, allow the user to press `d` (for example) to display a dialogue window that shows the payout for each valid number of shares, for full-pay and half-pay.

To do this, we need to know how many shares each company has.
This detail should presumably be stored in `n18game::Company`.

Note that tracking player and company assets, similar to [18SH](https://github.com/msaari/18sh), is a much more complex matter and not something I plan to address.

## UI navigation

Make, e.g., `g` open a text-entry widget and allow the user to enter a hex address to go to (i.e., make active).
Similarly, make `/` open a text-entry widget to select a replacement tile, allowing the user to filter matching tiles by typing their name or parts thereof.

Underlying this would be a modal window that accepts a slice of strings and allows the user to filter and select the desired option.
It seems that this might require using a `TreeModelFilter` to filter a `TreeModel` (such as `ListStore`, which appears sufficient), which is displayed using a `TreeView` widget.

The following references may be useful:

- [Python GTK+ 3 Tutorial](https://python-gtk-3-tutorial.readthedocs.io/en/latest/treeview.html)
- [GTK+ By Example](https://en.wikibooks.org/wiki/GTK%2B_By_Example/Tree_View/Tree_Models)
- [A StackOverflow question](https://stackoverflow.com/q/56029759)
