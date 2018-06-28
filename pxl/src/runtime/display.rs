//! Graphics rendering for the native OpenGl-based runtime

use runtime::common::*;

use runtime::gl;

static VERTICES: [GLfloat; 24] = [
  -1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 1.0, -1.0, 1.0, 0.0, 1.0, 1.0, -1.0,
  0.0, 1.0, 1.0, 1.0, 0.0, 1.0,
];

static DEFAULT_VERTEX_SHADER: &str = include_str!("../vertex_shader.glsl");

static DEFAULT_FRAGMENT_SHADER: &str = include_str!("../fragment_shader.glsl");

pub struct Display {
  shader_program: u32,
  pixel_texture: u32,
  passthrough_program: u32,
  filter_shader_programs: Vec<u32>,
  framebuffer_textures: Vec<u32>,
  framebuffers: Vec<u32>,
  vao: u32,
  vbo: u32,
  shader_cache: ShaderCache,
  frame: u64,
}

impl Display {
  pub fn new() -> Result<Display, Error> {
    unsafe {
      gl::ClearColor(0.0, 0.0, 0.0, 1.0);
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

    let mut pixel_texture = 0;
    unsafe {
      gl::GenTextures(1, &mut pixel_texture);
      gl::BindTexture(gl::TEXTURE_2D, pixel_texture);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    }

    let mut framebuffer_textures = vec![0, 0];
    unsafe {
      gl::GenTextures(
        framebuffer_textures.len() as i32,
        framebuffer_textures.as_mut_ptr(),
      );
      for texture in framebuffer_textures.iter().cloned() {
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
      }
    }

    let mut framebuffers = vec![0, 0];
    unsafe {
      gl::GenFramebuffers(framebuffers.len() as i32, framebuffers.as_mut_ptr());
    }

    for (framebuffer, texture) in framebuffers
      .iter()
      .cloned()
      .zip(framebuffer_textures.iter().cloned())
    {
      unsafe {
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
        gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, texture, 0);
        let draw_buffers: [u32; 1] = [gl::COLOR_ATTACHMENT0];
        gl::DrawBuffers(draw_buffers.len() as i32, (&draw_buffers).as_ptr());
      }
    }

    let mut shader_cache = ShaderCache::new();

    let passthrough_program =
      shader_cache.compile_program(DEFAULT_VERTEX_SHADER, DEFAULT_FRAGMENT_SHADER)?;

    Ok(Display {
      shader_program: 0,
      filter_shader_programs: Vec::new(),
      frame: 0,
      passthrough_program,
      pixel_texture,
      shader_cache,
      framebuffer_textures,
      framebuffers,
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
    self.shader_program = self
      .shader_cache
      .compile_program(vertex_shader_source, fragment_shader_source)?;

    self.filter_shader_programs = filter_shader_sources
      .iter()
      .map(|filter_shader_source| {
        self
          .shader_cache
          .compile_program(DEFAULT_VERTEX_SHADER, filter_shader_source)
      })
      .collect::<Result<Vec<u32>, Error>>()?;

    Ok(())
  }

  pub fn present(&mut self, pixels: &[Pixel], dimensions: (usize, usize), window_size: (u32, u32)) {
    let pixels = pixels.as_ptr();
    let bytes = pixels as *const c_void;

    let pass_count = self.filter_shader_programs.len() + 1;

    let mut input_index = 0;
    let mut output_index = 1;

    for pass in 0..pass_count {
      let first = pass == 0;

      let program = if first {
        self.shader_program
      } else {
        self.filter_shader_programs[pass - 1]
      };

      let input_texture = self.framebuffer_textures[input_index];
      let output_framebuffer = self.framebuffers[output_index];
      let output_texture = self.framebuffer_textures[output_index];

      unsafe {
        gl::UseProgram(program);

        gl::BindTexture(gl::TEXTURE_2D, output_texture);
        gl::TexImage2D(
          gl::TEXTURE_2D,
          0,
          gl::RGBA32F as i32,
          dimensions.0 as i32,
          dimensions.1 as i32,
          0,
          gl::RGBA,
          gl::FLOAT,
          0 as *const c_void,
        );

        if first {
          gl::BindTexture(gl::TEXTURE_2D, self.pixel_texture);
          gl::ActiveTexture(gl::TEXTURE0);
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
        } else {
          gl::BindTexture(gl::TEXTURE_2D, input_texture);
          gl::ActiveTexture(gl::TEXTURE0);
        }

        gl::BindFramebuffer(gl::FRAMEBUFFER, output_framebuffer);

        if self.frame == 0 {
          if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            panic!("Failed to prepare framebuffer");
          }
        }

        gl::Clear(gl::COLOR_BUFFER_BIT);
        gl::Viewport(0, 0, dimensions.0 as i32, dimensions.1 as i32);
        gl::DrawArrays(gl::TRIANGLES, 0, 6);
      }

      mem::swap(&mut input_index, &mut output_index);
    }

    let input_texture = self.framebuffer_textures[input_index];

    unsafe {
      gl::UseProgram(self.passthrough_program);
      gl::BindTexture(gl::TEXTURE_2D, input_texture);
      gl::ActiveTexture(gl::TEXTURE0);
      gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
      gl::Viewport(0, 0, window_size.0 as i32, window_size.1 as i32);
      gl::Clear(gl::COLOR_BUFFER_BIT);
      gl::DrawArrays(gl::TRIANGLES, 0, 6);
    }

    unsafe {
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
      gl::DeleteTextures(
        self.framebuffer_textures.len() as i32,
        self.framebuffer_textures.as_mut_ptr(),
      );
      gl::DeleteFramebuffers(
        self.framebuffers.len() as i32,
        self.framebuffers.as_mut_ptr(),
      );
      gl::DeleteBuffers(1, &self.vbo);
      gl::DeleteVertexArrays(1, &self.vao);
      assert_eq!(gl::GetError(), gl::NO_ERROR);
    }
  }
}
