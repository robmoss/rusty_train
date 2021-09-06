## Port to GTK 4

The [gtk-rs project](https://gtk-rs.org/) have released a [GTK 4 crate](https://crates.io/crates/gtk4) and an [introductory book](https://gtk-rs.org/gtk4-rs/stable/latest/book/).

It may be as simple as using the `gtk4` and `gdk4` crates, and making a few changes to the `rusty_train` binary.

Note that GTK 4 is not yet packages for Debian stable, testing, or unstable.
See [Debian bug 992907](https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=992907) for progress.
