#version 330 core

uniform sampler2D tex;

out vec4 color;
in vec2 position;

void main() {
    color = texture(tex, position);
}