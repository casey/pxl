mod runtime;

pub const SAMPLES_PER_SECOND: u32 = 48_000;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Pixel {
  pub red: u8,
  pub green: u8,
  pub blue: u8,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub enum Event {
  Button { state: ButtonState, button: Button },
  Key { character: char },
}

pub trait Program: Send {
  /// Initialize a new Program object
  fn new() -> Self
  where
    Self: Sized;

  /// Return the desired width and height of pixel surface
  ///
  /// Determines the length of the pixel slice passed to
  /// `render()`. If (256, 256) is returned, the pixel
  /// slice passed to `render()` will contain 256 * 256,
  /// elements.
  fn dimensions() -> (usize, usize)
  where
    Self: Sized;

  /// Return the vertex shader to be used in the runtime's
  /// rendering pipeline
  fn vertex_shader() -> String
  where
    Self: Sized,
  {
    include_str!("vertex_shader.glsl").to_string()
  }

  /// Return the fragment shader to be used in the runtime's
  /// rendering pipeline
  fn fragment_shader() -> String
  where
    Self: Sized,
  {
    include_str!("fragment_shader.glsl").to_string()
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
  /// * `pixels` — a 256 * 256 long slice of pixels. `pixels[x + y * 256]` is
  ///              the `x`th pixel in the `y`th row.
  fn render(&mut self, _pixels: &mut [Pixel]) {}

  /// Synthesize audio
  ///
  /// Called by the runtime as needed to fill the outgoing audio buffer
  ///
  /// * `played`  — number of samples written by previous calls to synthesize
  /// * `buffer`  — an array of audio samples
  fn synthesize(&mut self, _samples_played: u64, _buffer: &mut [Sample]) {}
}

pub fn run<P: Program + 'static>() -> ! {
  use runtime::Runtime;
  use std::{process,
            sync::{Arc, Mutex}};

  let program = P::new();
  let dimensions = P::dimensions();
  let vertex_shader = P::vertex_shader();
  let fragment_shader = P::fragment_shader();
  let result = Runtime::new(
    Arc::new(Mutex::new(program)),
    dimensions,
    &vertex_shader,
    &fragment_shader,
  ).and_then(|runtime| runtime.run());

  if let Err(error) = result {
    eprintln!("{}", error);
    process::exit(1);
  } else {
    process::exit(0);
  }
}
