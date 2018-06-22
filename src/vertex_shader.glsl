#version 150

in  vec4 position;
out vec2 uv;

void main() {
  uv = position.xy * 0.5 + vec2(0.5, 0.5);
  uv.y = 1 - uv.y;
  gl_Position = position;
}
