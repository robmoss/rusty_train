## Company shares and dividends

Allow defining players and companies who may comprise some number of shares, which can be owned by players, the company itself, etc.
Then the UI can automate the paying of dividends, allowing for things such as full-pay, half-pay, and withhold.

This would go some way to providing features akin to [18SH](https://github.com/msaari/18sh).
See the [BGG18SH thread](https://boardgamegeek.com/thread/2225619/18sh-command-line-replacement-spreadsheets) for ideas and planned features.
Complications include: rusting, nationalisation, mergers, loans, buying private companies and trains from other companies, etc.

- Having found the optimal routes for a company, what about being able to press `d` to distribute full dividends, `h` to half-pay dividends, and `w` to withhold?

- Having a game action log/console, into which players can enter commands and log games, undo actions, etc.

  For example, paying out dividends (or not) with any of the above commands from the UI could execute the appropriate command(s) in the console and log their output (e.g., pressing `d` could run a dividends command and also log a comment for each player who received money).
