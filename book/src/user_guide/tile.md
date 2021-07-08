# Replace tile mode

Use this mode to place and upgrade tiles.

| Key      | Action                                                       |
|----------|--------------------------------------------------------------|
| `Esc`    | Return to [**Default**](default.md) mode, ignoring any edits |
| `Return` | Return to [**Default**](default.md) mode, saving any edits   |
| `o`, `O` | Show the original tile, if any                               |
| `<Up>`   | Select the next available tile                               |
| `<Down>` | Select the previous available tile                           |
| `,`, `<` | Rotate the selected tile anti-clockwise                      |
| `.`, `>` | Rotate the selected tile clockwise                           |

Note that this mode allows the user to replace a tile with any available tile, and does not enforce any criteria for upgrade tiles.

This mode attempts to draw all of the tokens placed on the original tile, but may not do so even if the replacement tile has a sufficient number of token spaces.
Once a replacement tile has been selected, the user may need to manually place tokens on this tile.
