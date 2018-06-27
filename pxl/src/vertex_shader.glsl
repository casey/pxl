//! The default vertex shader
//! 
//! Takes as input the position in screen space of
//! a corner of a full-screen quad, which is used
//! to calculate and pass the texture coordinates
//! that fragment shaders should sample.
//!
//! You may ovverride the default vertex shader by
//! returning a custom shader in `Program::vertex_shader`.

// The version of the GLSL shading language to use.
// Custom shaders may set this to any value supported by
// the system.
//
// For maximum compatibility it may be desirable to use
// as low a version as possible
#version 150

// The input vertex. Will be one vertex of a full screen
// quad with the following coordinates:
//
//       X     Y    Z    W
// NW: -1.0,  1.0, 0.0, 1.0
// NE:  1.0,  1.0, 0.0, 1.0
// SW: -1.0, -1.0, 0.0, 1.0
// SE: -1.0,  1.0, 0.0, 1.0
//
// NW--NE
// |    |
// SW--SE
in vec4 position;

// Texture coordinate at the input vertex. The default
// fragment shader expects coordinates in the following space:
//
//      X     Y
// NW: 0.0, 0.0
// NE: 1.0, 0.0
// SW: 0.0, 1.0
// SE: 1.0, 1.0
//
// NW--NE
// |    |
// SW--SE
out vec2 uv;

// The vertex shader main function. Defines the entry point into
// the shader.
void main() {
  // Transform the vertex xy coordinates into the range 0.0-1.0
  uv = position.xy * 0.5 + vec2(0.5, 0.5);
  // Flip the y coordinate
  uv.y = 1 - uv.y;
  // Use the input vertex as-is as output from the vertex shader
  gl_Position = position;
}
