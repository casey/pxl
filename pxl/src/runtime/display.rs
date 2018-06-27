//! Graphics rendering for the native OpenGl-based runtime

use super::*;

use runtime::gl::types::*;

use std::collections::HashMap;

pub static VERTICES: [GLfloat; 24] = [
  -1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 1.0, -1.0, 1.0, 0.0, 1.0, 1.0, -1.0,
  0.0, 1.0, 1.0, 1.0, 0.0, 1.0,
];

pub struct Display {
  fragment_shader: u32,
  shader_program: u32,
  texture: u32,
  vao: u32,
  vbo: u32,
  vertex_shader: u32,
  vertex_shader_cache: HashMap<String, u32>,
  fragment_shader_cache: HashMap<String, u32>,
  shader_program_cache: HashMap<(u32, u32), u32>,
}

impl Display {
  pub fn new() -> Result<Display, Error> {
    let mut texture = 0;
    unsafe {
      gl::GenTextures(1, &mut texture);
      gl::BindTexture(gl::TEXTURE_2D, texture);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
      gl::ActiveTexture(gl::TEXTURE0);
    }

    let mut vao = 0;
    let mut vbo = 0;
    unsafe {
      gl::GenVertexArrays(1, &mut vao);
      gl::BindVertexArray(vao);

      // Create a Vertex Buffer Object and copy the vertex data to it
      gl::GenBuffers(1, &mut vbo);
      gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
      gl::BufferData(
        gl::ARRAY_BUFFER,
        (VERTICES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
        &VERTICES[0] as *const f32 as *const std::os::raw::c_void,
        gl::STATIC_DRAW,
      );
    }

    Ok(Display {
      vertex_shader_cache: HashMap::new(),
      fragment_shader_cache: HashMap::new(),
      shader_program_cache: HashMap::new(),
      fragment_shader: 0,
      shader_program: 0,
      vertex_shader: 0,
      texture,
      vao,
      vbo,
    })
  }

  pub fn set_shaders(
    &mut self,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
    filter_shader_sources: &[&str],
  ) -> Result<(), Error> {
    // Compile vertex shader
    let vertex_shader = Self::compile_shader(
      vertex_shader_source,
      gl::VERTEX_SHADER,
      &mut self.vertex_shader_cache,
    )?;

    // Compile fragment shader
    let fragment_shader = Self::compile_shader(
      fragment_shader_source,
      gl::FRAGMENT_SHADER,
      &mut self.fragment_shader_cache,
    )?;

    // Link shader program
    let shader_program = Self::link_program(
      vertex_shader,
      fragment_shader,
      vertex_shader_source,
      fragment_shader_source,
      &mut self.shader_program_cache,
    )?;

    // compile filter shaders
    for filter_shader_source in filter_shader_sources {
      // TODO: refactor this so it's a single function
      //       that takes all caches. or maybe create a
      //       shader cache object and make it a method
      //       on that.
      // TODO: do I need to set attributes/etc every time
      //       i switch programs, or just once the first
      //       time i load them?
      let filter_shader = Self::compile_shader(
        filter_shader_source,
        gl::FRAGMENT_SHADER,
        &mut self.fragment_shader_cache,
      )?;

      // Link filter shader program
      let _filter_shader_program = Self::link_program(
        vertex_shader,
        filter_shader,
        vertex_shader_source,
        filter_shader_source,
        &mut self.shader_program_cache,
      )?;
    }

    if self.shader_program != shader_program {
      unsafe {
        gl::UseProgram(shader_program);
        let zcolor = CString::new("color").unwrap();
        gl::BindFragDataLocation(shader_program, 0, zcolor.as_ptr());

        let zpixels = CString::new("pixels").unwrap();
        let pixel_uniform = gl::GetUniformLocation(shader_program, zpixels.as_ptr());
        gl::Uniform1i(pixel_uniform, 0);

        let zposition = CString::new("position").unwrap();
        let pos_attr = gl::GetAttribLocation(shader_program, zposition.as_ptr());
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(
          pos_attr as GLuint,
          4,
          gl::FLOAT,
          gl::FALSE as GLboolean,
          0,
          ptr::null(),
        );
      }

      self.shader_program = shader_program;
    }
    Ok(())
  }

  pub fn present(&self, pixels: &[Pixel], dimensions: (usize, usize)) {
    let pixels = pixels.as_ptr();
    let bytes = pixels as *const std::os::raw::c_void;

    unsafe {
      gl::ClearColor(0.0, 0.0, 0.0, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT);

      gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGBA32F as i32,
        dimensions.0 as i32,
        dimensions.1 as i32,
        0,
        gl::RGBA,
        gl::FLOAT,
        bytes,
      );

      gl::DrawArrays(gl::TRIANGLES, 0, 6);

      #[cfg(debug_assertions)]
      assert_eq!(gl::GetError(), gl::NO_ERROR);
    }
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

  fn link_program(
    vs: GLuint,
    fs: GLuint,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
    program_cache: &mut HashMap<(GLuint, GLuint), GLuint>,
  ) -> Result<GLuint, Error> {
    let cache_key = (vs, fs);

    if let Some(program) = program_cache.get(&cache_key).cloned() {
      return Ok(program);
    }

    unsafe {
      let program = gl::CreateProgram();
      gl::AttachShader(program, vs);
      gl::AttachShader(program, fs);
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

      program_cache.insert(cache_key, program);

      Ok(program)
    }
  }
}

impl Drop for Display {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteProgram(self.shader_program);
      gl::DeleteShader(self.fragment_shader);
      gl::DeleteShader(self.vertex_shader);
      gl::DeleteTextures(1, &self.texture);
      gl::DeleteBuffers(1, &self.vbo);
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}
