//! A test harness for `pxl-build`

extern crate pxl_build;

fn main() -> Result<(), pxl_build::Error> {
  pxl_build::build("resources")
}
