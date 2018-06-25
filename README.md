# pxl

A simple framework for making games in Rust.

## Building

Since `pxl` programs use per-pixel rendering, building in release mode can yield dramatically improved performance. Do `cargo run --release` to build in release mode.

## Dependencies

The `pxl` runtime plays audio using `cpal`, which requires ALSA headers/libraries on Linux;

On Ubuntu, you can install them with:

```sh
sudo apt install libasound2-dev
```
