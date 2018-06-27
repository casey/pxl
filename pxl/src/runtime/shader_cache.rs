use runtime::common::*;

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
}
