## Use a single index for token spaces

Token spaces are currently indexed by revenue centre and by the token space number in that revenue centre.
Using a flat index `0..N` instead (or in addition) could make other parts of the code simpler and easier to understand.
For example, this would make it much simpler to show all placed tokens on replacement tiles.
