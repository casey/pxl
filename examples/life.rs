extern crate pxl;
extern crate rand;

use std::{f64, sync::{Arc, Mutex}};
use rand::prelude::*;
use pxl::*;

use Cell::*;

const WIDTH:  usize = 1024;
const HEIGHT: usize = 1024;
const TAU:    f64   = f64::consts::PI * 2.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
  Alive,
  Dead,
}

impl Cell {
  pub fn tick(self, neighbors: u8) -> Cell {
    match (self, neighbors) {
      (Alive, 0) => Dead,
      (Alive, 1) => Dead,
      (Alive, 2) => Alive,
      (Alive, 3) => Alive,
      (Alive, _) => Dead,
      (Dead,  0) => Dead,
      (Dead,  1) => Dead,
      (Dead,  2) => Dead,
      (Dead,  3) => Alive,
      (Dead,  _) => Dead,
    }
  }
}

struct LifeSynthesizer {
  intensity: f32,
}

impl Synthesizer for LifeSynthesizer {
  fn synthesize(&mut self, mut samples_played: u64, samples: &mut [Sample]) {
    for sample in samples {
      let time = samples_played as f64 / SAMPLES_PER_SECOND as f64;
      let s = (time * 440.0 * TAU).sin() as f32 * self.intensity;
      sample.left = s;
      sample.right = s;
      samples_played += 1;
    }
  }
}

struct Life {
  synthesizer: Arc<Mutex<LifeSynthesizer>>,
  cells:       Vec<Cell>,
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

  pub fn reset(&mut self) {
    self.cells = (0..WIDTH*HEIGHT).into_iter().map(|_| if random() {
      Alive
    } else {
      Dead
    }).collect();
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
      synthesizer: Arc::new(Mutex::new(LifeSynthesizer {
        intensity: 0.0,
      })),
    }
  }

  fn dimensions(&self) -> (usize, usize) {
    (WIDTH, HEIGHT)
  }

  fn title(&self) -> &str {
    "life"
  }

  fn tick(&mut self, events: &[Event]) {
    for event in events {
      if let Event::Button{state: ButtonState::Released, button: Button::Action} = event {
        self.reset();
      }
    }

    self.step();

    let alive: f32 = self.cells.iter().map(|cell| match cell { Alive => 1.0, Dead => 0.0, }).sum();
    let intensity = alive / self.cells.len() as f32;

    self.synthesizer.lock().unwrap().intensity = intensity;
  }

  fn render(&mut self, pixels: &mut [Pixel]) {
    assert_eq!(pixels.len(), self.cells.len());
    for (pixel, cell) in pixels.iter_mut().zip(&self.cells) {
      *pixel = match cell {
        Alive => Pixel {
          red:   random(),
          green: random(),
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

  fn synthesizer(&self) -> Option<Arc<Mutex<Synthesizer>>> {
    Some(self.synthesizer.clone())
  }

  fn fragment_shader(&self) -> &str {
    "
#version 150

in  vec2 uv;
out vec4 color;

uniform sampler2D pixels;

vec2 barrel(vec2 coord, float amt) {
  vec2 cc = coord - 0.5;
  float dist = dot(cc, cc);
	return coord + cc * dist * amt;
}

float sat(float t) {
	return clamp(t, 0.0, 1.0);
}

float linterp(float t) {
	return sat( 1.0 - abs( 2.0*t - 1.0 ) );
}

float remap( float t, float a, float b ) {
	return sat( (t - a) / (b - a) );
}

vec3 spectrum_offset( float t ) {
	vec3 ret;
	float lo = step(t,0.5);
	float hi = 1.0-lo;
	float w = linterp( remap( t, 1.0/6.0, 5.0/6.0 ) );
	ret = vec3(lo,1.0,hi) * vec3(1.0-w, w, 1.0-w);

	return pow( ret, vec3(1.0/2.2) );
}

const float max_distort = 2.2;
const int num_iter = 12;
const float reci_num_iter_f = 1.0 / float(num_iter);

vec4 aberrate() {
	vec3 sumcol = vec3(0.0);
	vec3 sumw = vec3(0.0);
	for ( int i=0; i<num_iter;++i ) {
		float t = float(i) * reci_num_iter_f;
		vec3 w = spectrum_offset( t );
		sumw += w;
		sumcol += w * texture(pixels, barrel(uv, 0.5 * max_distort*t ) ).rgb;
	}

  return vec4(sumcol.rgb / sumw, 1.0);
}

void main() {
  color = aberrate();
}
"
  }

}

fn main() {
  pxl::run::<Life>();
}
