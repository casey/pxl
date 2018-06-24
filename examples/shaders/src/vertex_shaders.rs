pub static VERTEX_SHADERS: &[&str] = &[
// default vertex shader
r#"
#version 150

in  vec4 position;
out vec2 uv;

void main() {
  uv = position.xy * 0.5 + vec2(0.5, 0.5);
  uv.y = 1 - uv.y;
  gl_Position = position;
}
"#,
// vertical flip;
r#"
#version 150

in  vec4 position;
out vec2 uv;

void main() {
  uv = position.xy * 0.5 + vec2(0.5, 0.5);
  uv.y = uv.y;
  gl_Position = position;
}
"#,
// squish vertically
r#"
#version 150

in  vec4 position;
out vec2 uv;

void main() {
  uv = position.xy * 0.5 + vec2(0.5, 0.5);

  gl_Position = vec4(position.x, position.y * 0.5, position.zw);
}
"#,
];
