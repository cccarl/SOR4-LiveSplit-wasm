# Streets of Rage 4 AutoSplitter Remake

To be used with LiveSplit's new WASM runtime, to build you need to add to the rust toolchain:

* `$ rustup target add wasm32-unknown-unknown`

Recomemded to use cargo watch while developing:

* `cargo watch -x "build --target wasm32-unknown-unknown"`

To build for release:

* `$ cargo build --release --target wasm32-unknown-unknown`

You can add the WASM file in the target forlder to LiveSplit or the ASR Debugger.
