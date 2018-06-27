//! Runtime errors

use runtime::common::*;

use runtime::glutin;

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
    source: String,
    info_log: String,
  },
  FragmentShaderCompilation {
    source: String,
    info_log: String,
  },
  ShaderProgramLinking {
    vertex_shader_source: String,
    fragment_shader_source: String,
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

impl fmt::Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    use self::Error::*;
    match self {
      AudioOutputDeviceInitialization => write!(f, "Failed to initialize audio output device."),
      AudioOutputDoesNotSupport48khzSampleRate => {
        write!(f, "Audio output device does not support 48khz sample rate")
      }
      WindowCreation { creation_error } => write!(f, "Failed to create window: {}", creation_error),
      GraphicsContext { context_error } => {
        write!(f, "OpenGL graphics context errror: {}", context_error)
      }
      VertexShaderCompilation { source, info_log } => write!(
        f,
        "Failed to compile vertex shader. Source:\n{}\nInfo log:\n{}",
        source, info_log
      ),
      FragmentShaderCompilation { source, info_log } => write!(
        f,
        "Failed to compile fragment shader. Source:\n{}\nInfo log:\n{}",
        source, info_log
      ),
      ShaderProgramLinking {
        vertex_shader_source,
        fragment_shader_source,
        info_log,
      } => write!(
        f,
        "Failed to link shader program. Vertex shader source:\n{}
      Fragment shader source:\n{}\nInfo log:\n{}",
        fragment_shader_source, vertex_shader_source, info_log
      ),
    }
  }
}
