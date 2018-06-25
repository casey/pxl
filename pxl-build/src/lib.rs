#[cfg(test)]
extern crate tempfile;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quote;
extern crate image;
extern crate regex;
extern crate proc_macro;

mod error;
mod module;
mod resource;

use std::{env, path::Path, fs::File, io::prelude::*};

use error::Error;
use module::Module;

pub fn build(resource_directory: impl AsRef<Path>) -> Result<(), Error> {
  let module = Module::from_path(resource_directory)?;
  let tokens = module.tokens();
  let out_dir = env::var("OUT_DIR").unwrap();
  let dest_path = Path::new(&out_dir).join("resources.rs");
  let mut f = File::create(&dest_path).map_err(|io_error| (io_error, dest_path.as_path()))?;
  write!(f, "{}", tokens).map_err(|io_error| (io_error, dest_path.as_path()))?;
  Ok(())
}
