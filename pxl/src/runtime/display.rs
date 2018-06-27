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
        mem::transmute(&VERTICES[0]),
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
  ) -> Result<(), Error> {
    let vertex_shader = Self::compile_shader(
      vertex_shader_source,
      gl::VERTEX_SHADER,
      &mut self.vertex_shader_cache,
    ).map_err(|info_log| Error::VertexShaderCompilation { info_log })?;

    let fragment_shader = Self::compile_shader(
      fragment_shader_source,
      gl::FRAGMENT_SHADER,
      &mut self.fragment_shader_cache,
    ).map_err(|info_log| Error::FragmentShaderCompilation { info_log })?;

    let shader_program = Self::link_program(
      vertex_shader,
      fragment_shader,
      &mut self.shader_program_cache,
    ).map_err(|info_log| Error::ShaderProgramLinking { info_log })?;

    if self.shader_program != shader_program {
      unsafe {
        gl::UseProgram(shader_program);
        gl::BindFragDataLocation(shader_program, 0, CString::new("color").unwrap().as_ptr());

        let pixel_uniform =
          gl::GetUniformLocation(shader_program, CString::new("pixels").unwrap().as_ptr());
        gl::Uniform1i(pixel_uniform, 0);

        let pos_attr =
          gl::GetAttribLocation(shader_program, CString::new("position").unwrap().as_ptr());
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
  ) -> Result<GLuint, String> {
    if let Some(shader) = shader_cache.get(source).cloned() {
      return Ok(shader);
    }

    unsafe {
      let shader = gl::CreateShader(ty);
      let c_str = CString::new(source.as_bytes()).unwrap();
      gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
      gl::CompileShader(shader);

      // Get the compile status
      let mut status = gl::FALSE as GLint;
      gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

      // Fail on error
      if status != (gl::TRUE as GLint) {
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
        return Err(String::from_utf8_lossy(&buf).to_string());
      }

      shader_cache.insert(source.to_string(), shader);

      Ok(shader)
    }
  }

  fn link_program(
    vs: GLuint,
    fs: GLuint,
    program_cache: &mut HashMap<(GLuint, GLuint), GLuint>,
  ) -> Result<GLuint, String> {
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
      let mut status = gl::FALSE as GLint;
      gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

      // Fail on error
      if status != (gl::TRUE as GLint) {
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
        return Err(String::from_utf8_lossy(&buf).to_string());
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
