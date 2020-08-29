#version 330 core

#define or else if

out vec2 position;

void main() {
    vec2 p = vec2(0);
    if(gl_VertexID == 0) p = vec2(-1, -1);
    or(gl_VertexID == 1) p = vec2( 1, -1);
    or(gl_VertexID == 2) p = vec2( 1,  1);
    or(gl_VertexID == 3) p = vec2(-1, -1);
    or(gl_VertexID == 4) p = vec2( 1,  1);
    or(gl_VertexID == 5) p = vec2(-1,  1);
    position = p * .5 + .5;
    gl_Position = vec4(p, 0, 1);
}