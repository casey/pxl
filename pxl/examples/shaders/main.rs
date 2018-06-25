extern crate pxl;

mod fragment_shaders;
mod vertex_shaders;

use pxl::*;
use fragment_shaders::FRAGMENT_SHADERS;
use vertex_shaders::VERTEX_SHADERS;

const WIDTH:  usize = 768;
const HEIGHT: usize = 768;

const WHITE: Pixel = Pixel {
  red: 1.0,
  green: 1.0,
  blue: 1.0,
  alpha: 1.0,
};

const BLACK: Pixel = Pixel {
  red: 0.0,
  green: 0.0,
  blue: 0.0,
  alpha: 1.0,
};

struct Shaders {
  active_vertex_shader_index: usize,
  active_fragment_shader_index: usize,
}

impl Program for Shaders {
  fn new() -> Shaders {
    Shaders{active_vertex_shader_index: 0, active_fragment_shader_index: 0}
  }

  fn title(&self) -> &str {
    "pxl shaders"
  }

  fn dimensions(&self) -> (usize, usize) {
    (WIDTH, HEIGHT)
  }

  fn fragment_shader(&self) -> &str {
    FRAGMENT_SHADERS[self.active_fragment_shader_index]
  }

  fn vertex_shader(&self) -> &str {
    VERTEX_SHADERS[self.active_vertex_shader_index]
  }

  fn tick(&mut self, events: &[Event]) {
    for event in events {
      match event {
        Event::Button{state: ButtonState::Pressed, button: Button::Right} => {
          self.active_fragment_shader_index += 1;
        }
        Event::Button{state: ButtonState::Pressed, button: Button::Left} => {
          self.active_fragment_shader_index += FRAGMENT_SHADERS.len() - 1;
        }
        Event::Button{state: ButtonState::Pressed, button: Button::Up} => {
          self.active_vertex_shader_index += 1;
        }
        Event::Button{state: ButtonState::Pressed, button: Button::Down} => {
          self.active_vertex_shader_index += VERTEX_SHADERS.len() - 1;
        }
        _ => {}
      }
    }

    self.active_fragment_shader_index %= FRAGMENT_SHADERS.len();
    self.active_vertex_shader_index %= VERTEX_SHADERS.len();
  }

  fn render(&mut self, pixels: &mut [Pixel]) {
    let mut i = 0;
    for y in 0..HEIGHT {
      for x in 0..WIDTH {
        let row = x / 16;
        let col = y / 16;

        pixels[i] = if (col % 2 + row % 2) % 2 == 1 {
          WHITE
        } else {
          BLACK
        };

        i += 1;
      }
    }

  }
}

fn main() {
  pxl::run::<Shaders>();
}
