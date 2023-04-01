```shell
RUSTDOCFLAGS='--crate-version hurz -Z unstable-options --enable-index-page' rustup run nightly cargo doc --open --workspace --no-deps --all-features --locked
```