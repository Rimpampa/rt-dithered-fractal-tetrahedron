#version 330 core

uniform mat3 mat;

in vec3 coord;
out float depth;

const float angle = -.2;

const mat3 rot = mat3(
    1, 0, 0,
    0, cos(angle), sin(angle),
    0, -sin(angle), cos(angle)
);

void main(void) {
    vec3 pos = rot * mat * coord;
    pos.z += 1;
    pos.z *= .5;
    depth = 1 - pos.z;
    gl_Position = vec4(pos, 1);
}