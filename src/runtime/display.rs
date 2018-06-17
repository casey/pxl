use super::*;

use runtime::gl::types::*;

pub static VERTICES: [GLfloat; 24] = [
  -1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 0.0, 1.0, -1.0, -1.0, 0.0, 1.0, -1.0, 1.0, 0.0, 1.0, 1.0, -1.0,
  0.0, 1.0, 1.0, 1.0, 0.0, 1.0,
];

pub static VERTEX_SHADER_SOURCE: &'static str = include_str!("vertex.glsl");

pub static FRAGMENT_SHADER_SOURCE: &'static str = include_str!("fragment.glsl");

pub struct Display {
  fragment_shader: u32,
  shader_program: u32,
  texture: u32,
  vao: u32,
  vbo: u32,
  vertex_shader: u32,
  dimensions: (usize, usize),
}

impl Display {
  pub fn new(dimensions: (usize, usize)) -> Display {
    let vertex_shader = Self::compile_shader(VERTEX_SHADER_SOURCE, gl::VERTEX_SHADER);
    let fragment_shader = Self::compile_shader(FRAGMENT_SHADER_SOURCE, gl::FRAGMENT_SHADER);
    let shader_program = Self::link_program(vertex_shader, fragment_shader);

    let mut texture = 0;
    unsafe {
      gl::GenTextures(1, &mut texture);
      gl::BindTexture(gl::TEXTURE_2D, texture);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
      gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
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

      // Use shader program
      gl::UseProgram(shader_program);
      gl::BindFragDataLocation(shader_program, 0, CString::new("color").unwrap().as_ptr());

      let pixel_uniform =
        gl::GetUniformLocation(shader_program, CString::new("pixels").unwrap().as_ptr());
      gl::Uniform1i(pixel_uniform, 0);

      // Specify the layout of the vertex data
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

    Display {
      fragment_shader,
      shader_program,
      texture,
      vao,
      vbo,
      vertex_shader,
      dimensions,
    }
  }

  pub fn present(&self, pixels: &[Pixel]) {
    let pixels = pixels.as_ptr();
    let bytes = pixels as *const std::os::raw::c_void;

    unsafe {
      gl::ClearColor(0.0, 0.0, 0.0, 1.0);
      gl::Clear(gl::COLOR_BUFFER_BIT);

      gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::RGB as i32,
        self.dimensions.0 as i32,
        self.dimensions.1 as i32,
        0,
        gl::RGB,
        gl::UNSIGNED_BYTE,
        bytes,
      );

      gl::DrawArrays(gl::TRIANGLES, 0, 6);

      #[cfg(debug_assertions)]
      assert_eq!(gl::GetError(), gl::NO_ERROR);
    }
  }

  fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    unsafe {
      let shader = gl::CreateShader(ty);
      // Attempt to compile the shader
      let c_str = CString::new(src.as_bytes()).unwrap();
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
        panic!(
          "{}",
          str::from_utf8(&buf)
            .ok()
            .expect("ShaderInfoLog not valid utf8")
        );
      }
      shader
    }
  }

  fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
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
        panic!(
          "{}",
          str::from_utf8(&buf)
            .ok()
            .expect("ProgramInfoLog not valid utf8")
        );
      }
      program
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
