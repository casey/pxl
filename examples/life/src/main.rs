extern crate pxl;
extern crate rand;

mod cell;

use rand::prelude::*;
use pxl::*;

use cell::Cell::{self, *};

use std::f64;

const WIDTH:  usize = 512;
const HEIGHT: usize = 256;
const TAU:    f64   = f64::consts::PI * 2.0;

struct Life {
  cells: Vec<Cell>,
}

impl Life {
  fn index(&self, x: usize, y: usize) -> usize {
    x + y * WIDTH
  }

  fn neighbors(&self, i: usize) -> u8 {
    let mut neighbors = 0;

    let x = i % WIDTH;
    let y = (i - x) / WIDTH;

    let n = (y + HEIGHT - 1) % HEIGHT;
    let e = (x + 1) % WIDTH;
    let s = (y + 1) % HEIGHT;
    let w = (x + WIDTH - 1) % WIDTH;

    for y in &[n, y, s] {
      for x in &[w, x, e] {
        let ni = self.index(*x, *y);
      
        if ni == i {
          continue;
        }

        if self.cells[ni as usize] == Alive {
          neighbors += 1;
        }
      }
    }

    neighbors
  }

  pub fn step(&mut self) {
    let cells = self.cells.iter().enumerate().map(|(i, cell)| {
      cell.tick(self.neighbors(i))
    }).collect::<Vec<Cell>>();

    self.cells = cells;
  }
}

impl Program for Life {
  fn new() -> Life {
    Life {
      cells: (0..WIDTH*HEIGHT).into_iter().map(|_| if random() {
        Alive
      } else {
        Dead
      }).collect(),
    }
  }

  fn dimensions(&self) -> (usize, usize) {
    (WIDTH, HEIGHT)
  }

  fn title(&self) -> &str {
    "life"
  }

  fn tick(&mut self, _events: &[Event]) {
    self.step();
  }

  fn render(&mut self, pixels: &mut [Pixel]) {
    for (pixel, cell) in pixels.iter_mut().zip(&self.cells) {
      *pixel = match cell {
        Alive => Pixel {
          red:   random(),
          green: 0.0,
          blue:  random(),
          alpha: 1.0,
        },
        Dead => Pixel {
          red:   0.0,
          green: 0.0,
          blue:  0.0,
          alpha: 1.0,
        },
      };
    }
  }

  fn synthesize(&mut self, mut samples_played: u64, samples: &mut [Sample]) {
    let alive: f32 = self.cells.iter().map(|cell| match cell { Alive => 1.0, Dead => 0.0, }).sum();
    let intensity = alive / self.cells.len() as f32;

    for sample in samples {
      let time = samples_played as f64 / SAMPLES_PER_SECOND as f64;
      let s = (time * 440.0 * TAU).sin() as f32 * intensity;
      sample.left = s;
      sample.right = s;
      samples_played += 1;
    }
  }
}

fn main() {
  pxl::run::<Life>();
}
