pub static FRAGMENT_SHADERS: &[&str] = &[
// Passthrough
r#"
#version 150

in  vec2 uv;
out vec4 color;

uniform sampler2D pixels;

void main() {
  color = texture(pixels, uv);
}
"#,
// invert
r#"
#version 150

in  vec2 uv;
out vec4 color;

uniform sampler2D pixels;

void main() {
  color = vec4(vec3(1.0) - texture(pixels, uv).rgb, 1.0);
}
"#,

// CMYK
r#"
#version 150

in  vec2 uv;
out vec4 color;

uniform sampler2D pixels;

void main() {
  bool left = uv.x < 0.5;
  bool right = !left;
  bool up = uv.y < 0.5;
  bool down = !up;

  vec4 sample = texture(pixels, uv);

  if (up && left) {
    // cyan
    color = sample * vec4(0.0, 1.0, 1.0, 1.0);
  } else if (up && right) {
    // magenta
    color = sample * vec4(1.0, 0.0, 1.0, 1.0);
  } else if (down && left) {
    // yellow
    color = sample * vec4(1.0, 1.0, 0.0, 1.0);
  } else {
    // black
    color = sample * vec4(0.0, 0.0, 0.0, 1.0);
  }
}
"#,
// empty
r#"
#version 150
void main() {
}
"#,
// zoom
r#"
#version 150

in  vec2 uv;
out vec4 color;

uniform sampler2D pixels;

void main() {
  color = texture(pixels, uv / 2);
}
"#,
];
