## Undo/redo

Any UI event-handler that modifies the map should return an `Action` or `Command` enum that knows how to make **and** revert this modification to the map.
The UI can then maintain a vector of past actions and an index to the current undo position, allowing the user to undo and redo these actions.
Performing an action other than undo or redo would clear the future actions, and append this new action to the past actions.

The [Command pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/command.html) might be useful here.
Also see [these](https://redd.it/muei0l) [two](https://redd.it/mtknz0) `/r/rust` discussions about implementing undo/redo, and the [undo crate](https://github.com/evenorog/undo).
