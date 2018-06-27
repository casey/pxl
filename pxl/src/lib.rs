#![warn(missing_docs)]

mod runtime;

use std::sync::{Arc, Mutex};

pub const SAMPLES_PER_SECOND: u32 = 48_000;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Pixel {
  pub red: f32,
  pub green: f32,
  pub blue: f32,
  pub alpha: f32,
}

pub struct Image<'pixels> {
  pub width: usize,
  pub height: usize,
  pub pixels: &'pixels [Pixel],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sample {
  pub left: f32,
  pub right: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Button {
  Left,
  Right,
  Up,
  Down,
  Action,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ButtonState {
  Pressed,
  Released,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
  Button { state: ButtonState, button: Button },
  Key { character: char },
}

pub trait Synthesizer: Send + 'static {
  /// Synthesize audio
  ///
  /// Called by the runtime as needed to fill the outgoing audio buffer
  ///
  /// * `samples_played` — number of samples written by previous calls to synthesize
  /// * `output_buffer`  — the audio samples that synthesize should write
  fn synthesize(&mut self, _samples_played: u64, _output_buffer: &mut [Sample]) {}
}

pub trait Program: 'static {
  /// Initialize a new Program object
  fn new() -> Self
  where
    Self: Sized;

  /// Return the desired width and height of pixel surface
  ///
  /// Will be called immediately before calling `render()`.
  /// Determines the length of the pixel slice passed to
  /// `render()`. If (256, 256) is returned, the pixel
  /// slice passed to `render()` will contain 256 * 256,
  /// elements.
  fn dimensions(&self) -> (usize, usize);

  /// Return the vertex shader to be used in the runtime's
  /// rendering pipeline
  ///
  /// Will be called immediately before calling `render()`
  fn vertex_shader(&self) -> &str {
    include_str!("vertex_shader.glsl")
  }

  /// Return the fragment shader to be used in the runtime's
  /// rendering pipeline
  ///
  /// Will be called immediately before calling `render()`
  fn fragment_shader(&self) -> &str {
    include_str!("fragment_shader.glsl")
  }

  /// Return the title of the program
  ///
  /// Called by the runtime to set the window title
  fn title(&self) -> &str {
    "pxl"
  }

  /// Return true if the program should stop running
  ///
  /// Called by the runtime at the end of every pass through the event loop
  fn should_quit(&mut self) -> bool {
    false
  }

  /// Process events and update the state of the program
  ///
  /// Called by the runtime 60 times per second.
  ///
  /// * `events` — events that have occurred since the last call to `tick`
  fn tick(&mut self, _events: &[Event]) {}

  /// Draw to the display
  ///
  /// Called by the runtime whenever the display is ready to present a new frame
  ///
  /// WIDTH  — first element of the tuple returned by `dimensions()`
  /// HEIGHT — second element of the tuple returned by `dimensions()`
  ///
  /// * `pixels` — a slice of pixels with `WIDTH * HEIGHT` elements
  ///              `pixels[x + y * WIDTH]` is the `x`th pixel in the
  ///              `y`th row, with `(0,0)` starting in the upper left
  ///              corner of the screen
  fn render(&mut self, _pixels: &mut [Pixel]) {}

  /// The program's synthesizer
  ///
  /// Will be called by the runtime during initialization. If it returns
  /// Some, the contained Synthesizer will be moved to a dedicated audio
  /// thread and called periodically to produce samples for the outgoing
  /// audio stream.
  ///
  /// In order to prevent buffer underruns, avoid locking the `Mutex`
  /// containing the Synthesizer for long periods of time.
  fn synthesizer(&self) -> Option<Arc<Mutex<Synthesizer>>> {
    None
  }
}

pub fn run<P: Program>() -> ! {
  let program = P::new();
  let result = runtime::Runtime::new(Box::new(program)).and_then(|runtime| runtime.run());

  if let Err(error) = result {
    eprintln!("{}", error);
    std::process::exit(1);
  } else {
    std::process::exit(0);
  }
}
