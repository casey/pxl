extern crate pxl;

use pxl::*;

struct Blaster {}

impl Program for Blaster {
  fn new() -> Blaster {
    Blaster {}
  }

  fn resolution(&self) -> (usize, usize) {
    (256, 256)
  }

  // fn filter_shaders(&self) -> &[&str] {
  //   &[]
  // }

  fn render(&mut self, pixels: &mut [Pixel]) {
    for (i, pixel) in pixels.iter_mut().enumerate() {
      *pixel = if i % 2 == (i / 256) % 2 {
        rgb(0.0, 0.0, 0.0)
      } else {
        rgb(1.0, 1.0, 1.0)
      }
    }
  }
}

fn main() {
  run::<Blaster>()
}
