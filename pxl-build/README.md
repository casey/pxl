# pxl-build

`pxl-build` is a compile-time resource loader, intended for use with [`pxl`](https://github.com/casey/pxl).

## Usage

Add a folder called `resources` with your resources, and create a [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) with the following code:

```rust
extern crate pxl_build;

fn main() -> Result<(), pxl_build::Error> {
  pxl_build::build("resources")
}
```

This will create a rust source file containing your resources in `$OUT_DIR/resources.rs`, which can be used in your `pxl` program like so:

```rust
extern crate pxl;

include!(concat!(env!("OUT_DIR"), "/resources.rs"));
```

Your resources will then available with names derived from their filenames. 

For example, the image `resources/images/player.png` will be available in your program as `images::PLAYER`.

## Supported Resource Types

| File Type    | Extension(s)    | Rust Type             | Comments                               | 
| ------------ | --------------- | --------------------- | -------------------------------------- |
| Binary blobs | `.blob`         | `&[u8]`               | Arbitrary data exposed as a byte slice |
| PNG images   | `.png`          | `pxl::Image<'static>` | Lossless image format                  |
| JPEG images  | `.jpg`, `.jpeg` | `pxl::Image<'static>` | Lossy image format                     |


## Image Colorspace

`pxl` uses the rust image crate, which does not expose color space information. Image pixel data will be loaded as is appears on disk, without color space, linearity, or gamma correction. See issue #79 for details.
