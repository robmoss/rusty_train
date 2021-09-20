# Rusty Train

A graphical user interface for constructing 18xx maps and identifying optimal train routes.

## Usage

Start Rusty Train with `cargo run --release`:

![Rusty Train](./book/src/user_guide/1867_bc_0.png "Rusty Train")

Load the `1867_bc` example game, which is in `./examples/output/1867_bc.game`, with `Ctrl+O`:

![The 1867_bc example game](./book/src/user_guide/1867_bc_1.png "The 1867_bc example game")

Press `r` to find the optimal routes for a company.
Select the **Great Western Railway** and click `OK`:

![Select a company](./book/src/user_guide/1867_bc_2.png "Select a company")

This company owns a `5-train` and an `8-train`, and does not receive any of the four bonuses listed on the right-hand side.
Enter these details and click `OK`:

![Select trains](./book/src/user_guide/1867_bc_3.png "Select trains")

The map is disabled and faded out while searching for the optimal routes:

![Search for optimal routes](./book/src/user_guide/1867_bc_4.png "Search for optimal routes")

When the optimal routes are found, they will be drawn on the map (highlighted in green and in red) and the net revenue is shown in the window title:

![Found optimal routes](./book/src/user_guide/1867_bc_5.png "Found optimal routes")

Use the arrow keys (`<Left>`, `<Right>`, `<Up>`, `<Down>`) to cycle through the individual routes; the train name and route revenue are shown in the window title:

![Show a single route](./book/src/user_guide/1867_bc_6.png "Show a single route")

Press `Esc` or `Return` to return to the default user interface mode.

## License

Licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
