// The version of the GLSL shading language to use.
//
// Custom shaders may set this to any value supported by
// the system.
//
// For maximum compatibility it may be desirable to use
// as low a version as possible
#version 150

// The input texture coordinate at this fragment
in vec2 uv;

// The output color of this fragment
out vec4 color;

// A texture sampler containing the pixels written by the previous
// fragment shader in the pipeline
uniform sampler2D source;

void main() {
  // Look up the color in the `pixels` texture at coordinates `uv`
  // and use them to set the output color of this fragment.
  vec4 sample = texture(source, uv);
  
  color = vec4(vec3(1.0) - sample.rgb, sample.a);
}