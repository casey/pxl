extern crate glutin;

use std::fmt::{self, Display, Formatter};

pub enum Error {
  AudioOutputDeviceInitialization,
  AudioOutputDoesNotSupport48khzSampleRate,
  WindowCreation {
    creation_error: glutin::CreationError,
  },
  GraphicsContext {
    context_error: glutin::ContextError,
  },
  VertexShaderCompilation {
    info_log: String,
  },
  FragmentShaderCompilation {
    info_log: String,
  },
  ShaderProgramLinking {
    info_log: String,
  },
}

impl From<glutin::CreationError> for Error {
  fn from(creation_error: glutin::CreationError) -> Error {
    Error::WindowCreation { creation_error }
  }
}

impl From<glutin::ContextError> for Error {
  fn from(context_error: glutin::ContextError) -> Error {
    Error::GraphicsContext { context_error }
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use self::Error::*;
    match self {
      AudioOutputDeviceInitialization => write!(f, "Failed to initialize audio output device."),
      AudioOutputDoesNotSupport48khzSampleRate => write!(f, "Audio output device does not support 48khz sample rate"),
      WindowCreation { creation_error } => write!(f, "Failed to create window: {}", creation_error),
      GraphicsContext { context_error } => {
        write!(f, "OpenGL graphics context errror: {}", context_error)
      }
      VertexShaderCompilation { info_log } => {
        write!(f, "Failed to compile vertex shader:\n{}", info_log)
      }
      FragmentShaderCompilation { info_log } => {
        write!(f, "Failed to compile fragment shader:\n{}", info_log)
      }
      ShaderProgramLinking { info_log } => {
        write!(f, "Failed to link shader program:\n{}", info_log)
      }
    }
  }
}
