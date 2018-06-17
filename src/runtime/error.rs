extern crate glutin;

use std::fmt::{self, Display, Formatter};

pub enum Error {
  AudioOutputDeviceInitialization,
  WindowCreation(glutin::CreationError),
  GraphicsContext(glutin::ContextError),
}

impl From<glutin::CreationError> for Error {
  fn from(creation_error: glutin::CreationError) -> Error {
    Error::WindowCreation(creation_error)
  }
}

impl From<glutin::ContextError> for Error {
  fn from(context_error: glutin::ContextError) -> Error {
    Error::GraphicsContext(context_error)
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use self::Error::*;
    match self {
      AudioOutputDeviceInitialization => write!(f, "Failed to initialize audio output device."),
      WindowCreation(creation_error) => write!(f, "Failed to create window: {}", creation_error),
      GraphicsContext(context_error) => {
        write!(f, "OpenGL graphics context errror: {}", context_error)
      }
    }
  }
}
