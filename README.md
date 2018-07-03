# pxl

A simple framework for making games in Rust.

## Crates

`pxl` consists of a number of crates:

- `pxl`: The library and runtime. Start here!
- `pxl-build`: A compile-time resource loader. Check this out if you want to use static assets, like images and sounds.
- `pxl-build-test`: Tests and usage example for `pxl-build`.

## Building

Since `pxl` programs use per-pixel rendering, building in release mode can yield dramatically improved performance. Do `cargo run --release` to build in release mode.

## Dependencies

The `pxl` runtime plays audio using `cpal`, which requires ALSA headers/libraries on Linux;

On Ubuntu, you can install them with:

```sh
sudo apt install libasound2-dev
```

## Examples

To run one of the examples, type the following:
```sh
cargo run --package pxl --release --example EXAMPLE_NAME
```

To add one of your own examples, place it in pxl/examples and modify pxl/Cargo.toml
to include it.
