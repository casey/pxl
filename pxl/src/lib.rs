#![warn(missing_docs)]

//! # pxl
//!
//! A simple framework for making graphical programs in Rust. `pxl` is intended to avoid
//! Rust's most challenging concepts, while still providing a compelling platform upon
//! which to develop graphical games and programs.
//!
//! ## Features
//!
//! - Pixel-based rendering
//! - Sample-based audio synthesis
//! - Custom vertex and fragment shaders
//! - `pxl-build`, a compile-time resource loading crate
//! - Action and text input

mod runtime;

use std::sync::{Arc, Mutex};

/// The number of audio samples in a second. Synthesizer
/// implementations will need this to calculate the current
/// time from the number of samples played so far.
pub const SAMPLES_PER_SECOND: u32 = 48_000;

/// An RGBA pixel. Components should normally be between
/// `0.0` and `1.0` inclusive.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Pixel {
  /// The red component
  pub red: f32,
  /// The green component
  pub green: f32,
  /// The blue component
  pub blue: f32,
  /// The alpha component. Currently ignored, but in the
  /// future, pixels rendered with an alpha value of less
  /// than 1.0 may be transparent, exposing the desktop or
  /// window behind the `pxl` window.
  pub alpha: f32,
}

/// An image made of pixels. Used by the `pxl-build` crate
/// for image resources
pub struct Image<'pixels> {
  /// Width in pixels of the image
  pub width: usize,
  /// Height in pixels of the image
  pub height: usize,
  /// Pixels of the image, containing `width*height` pixels
  pub pixels: &'pixels [Pixel],
}

/// A single stereo audio sample, representing `1/SAMPLES_PER_SECOND`
/// of an audio signal
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sample {
  /// The left channel value
  pub left: f32,
  /// The right channel value
  pub right: f32,
}

/// Enum representing input buttons
///
/// In the current runtime, the arrow keys produce `Left`, `Right`, `Up` and
/// `Down` events, and the spacebar produces `Action` events.
///
/// Buttons are intended to be abstract, and in the future gamepad,
/// touch, and/or keyboard input may produce Button events
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Button {
  /// Left button
  Left,
  /// Right button
  Right,
  /// Up button
  Up,
  /// Down button
  Down,
  /// Action button
  Action,
}

/// Enum representing the state of an input button
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ButtonState {
  /// The button was pressed
  Pressed,
  /// The button was released
  Released,
}

/// Input events
///
/// Note that a single keyboard press may generate both
/// a `Button` and `Key` event. For example, the spacebar
/// will generate both a `Button::Action` event and a
/// `Key{character: ' '}` event.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Event {
  /// A button event, representing a pressed or released button
  Button {
    /// The button in question
    button: Button,
    /// The state of the button: `Pressed` or `Released`
    state: ButtonState,
  },
  /// A text input event
  Key {
    /// The input character
    character: char,
  },
}

/// Trait for things that can generate sound
///
/// When a computer program plays audio, it typically generates audio samples that
/// are fed to a DAC, a digital-to-analog converter. This is highly latency-sensitive:
/// audible pops and crackles may be heard if the program does not generate samples
/// quickly enough.
///
/// A `pxl::Program` may not be available to generate samples at all times, for example
/// if it is busy rendering. To make sure that samples for the underlying audio hardware
/// can be generated at all times, even during rendering, audio-sample synthesis is delegated
/// to a dedicated `Synthesizer` object.  
///
/// A `pxl::Program` that wishes to play audio should return an `Arc<Mutex<Synthesizer>>`
/// from `Program::synthesizer`. `Synthesizer::synthesize` will be called as needed on the
/// returned `Sythesizer` object to generate samples to feed the underlying audio hardware.
///
/// To communicate with and update the `Synthesizer`, the `pxl::Program` implementation
/// should keep it's own `Arc<Mutex<Synthesizer>>` with a reference to the synthesizer
/// object, locking it and updating it as needed.
///
/// In order to avoid starving the audio hardware of samples, the lock holding the
/// synthesizer should only be kept locked briefly.
pub trait Synthesizer: Send + 'static {
  /// Synthesize audio
  ///
  /// Called by the runtime as needed to fill the outgoing audio buffer
  ///
  /// * `samples_played` — number of samples written by previous calls to synthesize
  /// * `output_buffer`  — the audio samples that synthesize should write
  fn synthesize(&mut self, _samples_played: u64, _output_buffer: &mut [Sample]) {}
}

/// Trait representing a `pxl::Program`
///
/// To run a program, see the `run` function below.
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

/// Run a `pxl::Program`. `run` takes care of instantiating your
/// program, so your program is passed as a type parameter, not
/// as a value.
///
/// For example, to get a 256 by 256 black window that does nothing,
/// aside from being quietly awesome, do:
///
/// ```no_run
/// // Import the pxl create
/// extern crate pxl;
///
/// // Bring public members of pxl into scope
/// use pxl::*;
///
/// // Create an empty stuct to hold your program state
/// struct AwesomeProgram {
/// }
///
/// // Implement `pxl::Program` for your program. Only
/// // a few methods are required.
/// impl Program for AwesomeProgram {
///   // Create a new `AwesomeProgram`
///   fn new() -> AwesomeProgram {
///     AwesomeProgram{}
///   }
///
///   // Set the dimensions of the window
///   fn dimensions(&self) -> (usize, usize) {
///     (256, 256)
///   }
/// }
///
/// // The crate's main function
/// fn main() {
///   // Run the program, using the beloved turbofish syntax to
///   // indicate to the runtime which type of program to be run
///   run::<AwesomeProgram>();
/// }
/// ```
///
/// Once you have this working, you can start extending it
/// by implementing the various methods in `pxl::Program`.
///
/// Good luck and have fun!
///
/// <3,
/// Casey
pub fn run<P: Program>() -> ! {
  // Instantiate a new program
  let program = P::new();
  // Construct a new runtime
  let result = runtime::Runtime::new(Box::new(program));

  // Run the program
  if let Err(error) = result.and_then(|runtime| runtime.run()) {
    // Print an error message if something went wrong
    eprintln!("{}", error);
    // Terminate with an status code indicating that something went wrong
    std::process::exit(1);
  } else {
    // Terminate with an status code indicating that nothing went wrong
    std::process::exit(0);
  }
}
