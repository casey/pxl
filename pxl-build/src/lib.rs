#![warn(missing_docs)]

//! # `pxl-build`
//!
//! A compile time resource-loader for `pxl`.
//!
//! To use, add `pxl-build` to your build-dependencies, put all the
//! resources you would like to use in your `pxl` program in a directory
//! named `resources` in the root of your crate, and put the following
//! into a file named `build.rs`, also in the root of your crate:
//!
//! ```no_run
//! extern crate pxl_build;
//!
//! fn main() -> Result<(), pxl_build::Error> {
//!   pxl_build::build("resources")
//! }
//! ```
//!
//! Then, in your program, you can do:
//!
//! ```rust,ignore
//! extern crate pxl;
//!
//! include!(concat!(env!("OUT_DIR"), "/resources.rs"));
//! ```
//!
//! Your resources will be available as statics with uppercase
//! names derived from their filenames, in modules according to
//! the structure of the `resources` folder.
//!
//! For example, if you put an image into `resources/images/player.png`,
//! it will be available in your program as `images::PLAYER`.

#[cfg(test)]
extern crate tempfile;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quote;
extern crate image;
extern crate palette;
extern crate proc_macro2;
extern crate regex;
extern crate syn;

mod common;
mod error;
mod module;
mod resource;

pub use error::Error;

use common::*;

/// Compile the resources in `resource_directory` into `$OUT_DIR/resources.rs`
/// for inclusion in a `pxl` program.
pub fn build(resource_directory: impl AsRef<Path>) -> Result<(), Error> {
  let module = Module::from_path(resource_directory)?;
  let tokens = module.tokens();
  let out_dir = env::var("OUT_DIR").unwrap();
  let dest_path = Path::new(&out_dir).join("resources.rs");
  let mut f = File::create(&dest_path).map_err(|io_error| (io_error, dest_path.as_path()))?;
  write!(f, "{}", tokens).map_err(|io_error| (io_error, dest_path.as_path()))?;
  Ok(())
}
