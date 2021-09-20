# Example

Start Rusty Train with `cargo run --release`:

![Rusty Train](1867_bc_0.png "Rusty Train")

Load the `1867_bc` example game, which is in `./examples/output/1867_bc.game`, with `Ctrl+O`:

![The 1867_bc example game](1867_bc_1.png "The 1867_bc example game")

Press `r` to find the optimal routes for a company.
Select the **Great Western Railway** and click `OK`:

![Select a company](1867_bc_2.png "Select a company")

This company owns a `5-train` and an `8-train`, and does not receive any of the four bonuses listed on the right-hand side.
Enter these details and click `OK`:

![Select trains](1867_bc_3.png "Select trains")

The map is disabled and faded out while searching for the optimal routes:

![Search for optimal routes](1867_bc_4.png "Search for optimal routes")

When the optimal routes are found, they will be drawn on the map (highlighted in green and in red) and the net revenue is shown in the window title:

![Found optimal routes](1867_bc_5.png "Found optimal routes")

Use the arrow keys (`<Left>`, `<Right>`, `<Up>`, `<Down>`) to cycle through the individual routes; the train name and route revenue are shown in the window title:

![Show a single route](1867_bc_6.png "Show a single route")

Press `Esc` or `Return` to return to the [**Default**](default.md) mode.
