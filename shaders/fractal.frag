#version 330 core
uniform vec3 color;

out vec4 frag_color;
in float depth;

void main() {
    frag_color = vec4(color * depth, depth);
}