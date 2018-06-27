//! Graphics rendering for the native OpenGl-based runtime

use runtime::common::*;

use runtime::gl;

pub static VERTICES: [GLfloat; 24] = [
  -1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 1.0, -1.0, 1.0, 0.0, 1.0, 1.0, -1.0,
  0.0, 1.0, 1.0, 1.0, 0.0, 1.0,
];

pub struct Display {
  shader_program: u32,
  texture: u32,
  vao: u32,
  vbo: u32,
  shader_cache: ShaderCache,
  frame: u64,
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
        &VERTICES[0] as *const f32 as *const c_void,
        gl::STATIC_DRAW,
      );
    }

    Ok(Display {
      shader_cache: ShaderCache::new(),
      shader_program: 0,
      frame: 0,
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
    let shader_program = self
      .shader_cache
      .compile_program(vertex_shader_source, fragment_shader_source)?;

    if shader_program != self.shader_program {
      unsafe {
        gl::UseProgram(shader_program);
      }
      self.shader_program = shader_program;
    }

    for _filter_shader_source in filter_shader_sources {
      // TODO: WHICH vertex shader?
    }

    Ok(())
  }

  pub fn present(&mut self, pixels: &[Pixel], dimensions: (usize, usize)) {
    let pixels = pixels.as_ptr();
    let bytes = pixels as *const c_void;

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

      if self.frame == 0 {
        assert_eq!(gl::GetError(), gl::NO_ERROR);
      }
    }

    self.frame += 1;
  }
}

impl Drop for Display {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteTextures(1, &self.texture);
      gl::DeleteBuffers(1, &self.vbo);
      gl::DeleteVertexArrays(1, &self.vao);
      assert_eq!(gl::GetError(), gl::NO_ERROR);
    }
  }
}
