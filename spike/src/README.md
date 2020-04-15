
Issues:

1. It generates the `cloudstate.eventsourced.rs` filename that is not supported by Rust.
    It can't be imported with `mod cloudstate.eventsourced.rs` and needs to be renamed.

2. `cloudstate.eventsourced.rs` contains references to `cloudstate.rs`.
    How to make them visible? Using `mod cloudstate;` in `cloudstate.eventsourced.rs` didn't help.