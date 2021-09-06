## UI navigation

Make, e.g., `g` open a text-entry widget and allow the user to enter a hex address to go to (i.e., make active).
Similarly, make `/` open a text-entry widget to select a replacement tile, allowing the user to filter matching tiles by typing their name or parts thereof.

Underlying this would be a modal window that accepts a slice of strings and allows the user to filter and select the desired option.
It seems that this might require using a `TreeModelFilter` to filter a `TreeModel` (such as `ListStore`, which appears sufficient), which is displayed using a `TreeView` widget.

The following references may be useful:

- [Python GTK+ 3 Tutorial](https://python-gtk-3-tutorial.readthedocs.io/en/latest/treeview.html)
- [GTK+ By Example](https://en.wikibooks.org/wiki/GTK%2B_By_Example/Tree_View/Tree_Models)
- [A StackOverflow question](https://stackoverflow.com/q/56029759)
