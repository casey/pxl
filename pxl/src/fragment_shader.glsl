#version 150

in  vec2 uv;
out vec4 color;

uniform sampler2D pixels;

void main() {
  color = texture(pixels, uv);
}
