use runtime::{common::*, gl};

pub struct ShaderCache {
  vertex_shader_cache: HashMap<String, u32>,
  fragment_shader_cache: HashMap<String, u32>,
  shader_program_cache: HashMap<(u32, u32), u32>,
}

impl ShaderCache {
  pub fn new() -> ShaderCache {
    ShaderCache {
      vertex_shader_cache: HashMap::new(),
      fragment_shader_cache: HashMap::new(),
      shader_program_cache: HashMap::new(),
    }
  }

  pub fn compile_program(
    &mut self,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
  ) -> Result<GLuint, Error> {
    let vertex_shader = self.compile_vertex_shader(vertex_shader_source)?;
    let fragment_shader = self.compile_fragment_shader(fragment_shader_source)?;

    let cache_key = (vertex_shader, fragment_shader);

    if let Some(program) = self.shader_program_cache.get(&cache_key).cloned() {
      return Ok(program);
    }

    unsafe {
      let program = gl::CreateProgram();
      gl::AttachShader(program, vertex_shader);
      gl::AttachShader(program, fragment_shader);
      gl::LinkProgram(program);
      // Get the link status
      let mut status = GLint::from(gl::FALSE);
      gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

      // Fail on error
      if status != GLint::from(gl::TRUE) {
        let mut len: GLint = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
        gl::GetProgramInfoLog(
          program,
          len,
          ptr::null_mut(),
          buf.as_mut_ptr() as *mut GLchar,
        );
        let info_log = String::from_utf8_lossy(&buf).to_string();
        let vertex_shader_source = vertex_shader_source.to_string();
        let fragment_shader_source = fragment_shader_source.to_string();
        return Err(Error::ShaderProgramLinking {
          info_log,
          vertex_shader_source,
          fragment_shader_source,
        });
      }

      gl::UseProgram(program);
      let zcolor = CString::new("color").unwrap();
      gl::BindFragDataLocation(program, 0, zcolor.as_ptr());

      let zpixels = CString::new("source").unwrap();
      let pixel_uniform = gl::GetUniformLocation(program, zpixels.as_ptr());
      gl::Uniform1i(pixel_uniform, 0);

      let zposition = CString::new("position").unwrap();
      let pos_attr = gl::GetAttribLocation(program, zposition.as_ptr());
      gl::EnableVertexAttribArray(pos_attr as GLuint);

      gl::VertexAttribPointer(
        pos_attr as GLuint,
        4,
        gl::FLOAT,
        gl::FALSE as GLboolean,
        0,
        ptr::null(),
      );

      self.shader_program_cache.insert(cache_key, program);

      Ok(program)
    }
  }

  fn compile_vertex_shader(&mut self, shader_source: &str) -> Result<GLuint, Error> {
    Self::compile_shader(
      shader_source,
      gl::VERTEX_SHADER,
      &mut self.vertex_shader_cache,
    )
  }

  fn compile_fragment_shader(&mut self, shader_source: &str) -> Result<GLuint, Error> {
    Self::compile_shader(
      shader_source,
      gl::FRAGMENT_SHADER,
      &mut self.fragment_shader_cache,
    )
  }

  fn compile_shader(
    source: &str,
    ty: GLenum,
    shader_cache: &mut HashMap<String, GLuint>,
  ) -> Result<GLuint, Error> {
    if let Some(shader) = shader_cache.get(source).cloned() {
      return Ok(shader);
    }

    unsafe {
      let shader = gl::CreateShader(ty);
      let c_str = CString::new(source.as_bytes()).unwrap();
      gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
      gl::CompileShader(shader);

      // Get the compile status
      let mut status = GLint::from(gl::FALSE);
      gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

      // Fail on error
      if status != (GLint::from(gl::TRUE)) {
        let mut len = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        let mut buf = Vec::with_capacity(len as usize);
        buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
        gl::GetShaderInfoLog(
          shader,
          len,
          ptr::null_mut(),
          buf.as_mut_ptr() as *mut GLchar,
        );
        let info_log = String::from_utf8_lossy(&buf).to_string();
        let source = source.to_string();
        return Err(if ty == gl::FRAGMENT_SHADER {
          Error::FragmentShaderCompilation { source, info_log }
        } else {
          Error::VertexShaderCompilation { source, info_log }
        });
      }

      shader_cache.insert(source.to_string(), shader);

      Ok(shader)
    }
  }
}

impl Drop for ShaderCache {
  fn drop(&mut self) {
    unsafe {
      for shader in self.vertex_shader_cache.values().cloned() {
        gl::DeleteShader(shader);
      }

      for shader in self.fragment_shader_cache.values().cloned() {
        gl::DeleteShader(shader);
      }

      for program in self.shader_program_cache.values().cloned() {
        gl::DeleteProgram(program);
      }
    }
  }
}
