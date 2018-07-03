extern crate cpal;
extern crate pxl;
#[macro_use]
extern crate log;

#[macro_use]
extern crate num_derive;
extern crate num_traits;

use cpal::{EventLoop, Format, Sample, SampleRate, StreamData, SupportedFormat};

use std::{
  collections::VecDeque, sync::{Arc, Mutex}, thread,
};

use num_traits::FromPrimitive;

use pxl::*;

struct Loopback {
  samples: Arc<Mutex<VecDeque<AudioSample>>>,
}

const USE_LOOPBACK_DEVICE: bool = true;
const BLACK: Pixel = Pixel {
  red: 0.0,
  green: 0.0,
  blue: 0.0,
  alpha: 1.0,
};
const WHITE: Pixel = Pixel {
  red: 1.0,
  green: 1.0,
  blue: 1.0,
  alpha: 1.0,
};
const SIZE: usize = 256;

fn mono(white: bool) -> Pixel {
  if white {
    WHITE
  } else {
    BLACK
  }
}

impl Loopback {
  fn new() -> Loopback {
    let device = if USE_LOOPBACK_DEVICE {
      cpal::input_devices()
        .find(|device| {
          warn!("{}", device.name());
          device.name().contains("<null>")
        })
        .expect("Failed to find loopback device.")
    } else {
      cpal::default_input_device().unwrap()
    };

    let sample_rate = SampleRate(SAMPLES_PER_SECOND);
    let channels = 2;

    let mut supported_input_formats = device
      .supported_input_formats()
      .unwrap()
      .filter(|f| {
        f.channels == channels
          && f.min_sample_rate <= sample_rate
          && f.max_sample_rate >= sample_rate
      })
      .collect::<Vec<SupportedFormat>>();

    supported_input_formats.sort_unstable_by(|a, b| a.cmp_default_heuristics(b));

    let supported_input_format = supported_input_formats.first().unwrap();

    let input_format = Format {
      data_type: supported_input_format.data_type,
      channels,
      sample_rate,
    };

    let event_loop = EventLoop::new();

    let stream_id = event_loop
      .build_input_stream(&device, &input_format)
      .unwrap();

    event_loop.play_stream(stream_id);

    let samples = Arc::new(Mutex::new(VecDeque::new()));

    {
      let samples = samples.clone();

      thread::spawn(move || {
        event_loop.run(move |_stream_id, stream_data| {
          if let StreamData::Input { buffer } = stream_data {
            use cpal::UnknownTypeInputBuffer::*;
            let mut samples = samples.lock().unwrap();
            match buffer {
              F32(mut buffer) => {
                for i in 0..buffer.len() / 2 {
                  let i = i * 2;
                  samples.push_back(AudioSample {
                    left: buffer[i],
                    right: buffer[i + 1],
                  });
                }
              }
              I16(mut buffer) => {
                for i in 0..buffer.len() / 2 {
                  let i = i * 2;
                  samples.push_back(AudioSample {
                    left: buffer[i].to_f32(),
                    right: buffer[i + 1].to_f32(),
                  });
                }
              }
              U16(mut buffer) => {
                for i in 0..buffer.len() / 2 {
                  let i = i * 2;
                  samples.push_back(AudioSample {
                    left: buffer[i].to_f32(),
                    right: buffer[i + 1].to_f32(),
                  });
                }
              }
            }
          } else {
            panic!("unexpected output buffer");
          }
        });
      });
    }

    Loopback { samples }
  }
}

impl Synthesizer for Loopback {
  fn synthesize(&mut self, _samples_played: u64, samples: &mut [AudioSample]) {
    let mut incoming_samples = self.samples.lock().unwrap();
    for sample in samples {
      if let Some(input) = incoming_samples.pop_front() {
        *sample = input
      } else {
        sample.left = 0.0;
        sample.right = 0.0;
      }
    }
  }
}

#[derive(Copy, Clone, FromPrimitive)]
enum Pattern {
  Black,
  White,
  Grey,
  Striped,
  // Spiral,
  // Stippled,
  // Crosshatch,
}

impl Pattern {
  fn cycle(self) -> Pattern {
    FromPrimitive::from_u8(self as u8 + 1).unwrap_or_else(|| FromPrimitive::from_u8(0).unwrap())
  }

  fn render(self, pixels: &mut [Pixel]) {
    use Pattern::*;
    match self {
      Black => pixels.iter_mut().map(|pixel| *pixel = BLACK).count(),
      White => pixels.iter_mut().map(|pixel| *pixel = WHITE).count(),
      Grey => pixels
        .iter_mut()
        .enumerate()
        .map(|(i, pixel)| {
          let y = i / SIZE;
          *pixel = mono(i % 2 == y % 2);
        })
        .count(),
      Striped => pixels
        .iter_mut()
        .enumerate()
        .map(|(i, pixel)| {
          let y = i % SIZE;
          *pixel = mono(y % 2 == 0);
        })
        .count(),
    };
  }
}

struct Mono {
  pattern: Pattern,
}

impl Program for Mono {
  fn new() -> Mono {
    Mono {
      pattern: Pattern::Striped,
    }
  }

  fn resolution(&self) -> (usize, usize) {
    (SIZE, SIZE)
  }

  fn title(&self) -> &str {
    "mono|chrome"
  }

  fn fragment_shader(&self) -> &str {
    r#"
#version 150

in vec2 uv;

out vec4 color;

uniform sampler2D source;

uniform sampler2D samples;

uniform sampler2D frequencies;

vec4 render(float position, float intensity) {
  vec4 src = texture(source, uv);
  if (position < intensity) {
    return vec4(1.0 - src.rgb, src.a);
  } else {
    return src;
  }
}

void main() {
  vec4 sample = uv.y < 0.5 ? texture(samples, uv.ts) : texture(frequencies, uv.ts);

  if (uv.x < 0.5) {
    color = render(uv.x * 4 - 1, sample.x);
  } else {
    color = render(-(uv.x * 4 - 3), sample.y);
  }
}
"#
  }

  fn tick(&mut self, _: Duration, events: &[Event]) {
    for event in events {
      if let Event::Button {
        button: Button::Action,
        state: ButtonState::Pressed,
      } = event
      {
        self.pattern = self.pattern.cycle();
      }
    }
  }

  fn synthesizer(&self) -> Option<Arc<Mutex<Synthesizer>>> {
    Some(Arc::new(Mutex::new(Loopback::new())))
  }

  fn render(&mut self, pixels: &mut [Pixel]) {
    self.pattern.render(pixels);
  }
}

fn main() {
  run::<Mono>()
}
