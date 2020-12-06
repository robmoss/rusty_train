# GTK user interface

The `rusty_train` program uses (synchronous) channels to pass messages to a single event-handler that owns and mutates the UI state (`navig18xx::prelude::UI`).
The map state is drawn on an off-screen image surface, and whenever the event-handler determines that the user interface needs to be redrawn, it copies the contents of this off-screen surface to the on-screen drawing area.

I had previously tried using a `Rc<RefCell<UI>>` value to share a single mutable UI, but this [isn't a great idea](https://mmstick.keybase.pub/rust-gtk-practices/); message passing is a [much nicer alternative](https://coaxion.net/blog/2019/02/mpsc-channel-api-for-painless-usage-of-threads-with-gtk-in-rust/).

I also tried drawing the updated map state to a [recording surface](https://gtk-rs.org/docs/cairo/struct.RecordingSurface.html) and then copying this content to the on-screen drawing area, but this proved to be **extremely** slow.
Switching from a recording surface to a plain image surface resolved this issue, because the recording surface records each operation at the **most abstract level** and then replays them one by one.
