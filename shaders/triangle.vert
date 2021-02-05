#version 450

layout (location = 0) in vec3 i_position;
layout (location = 1) in vec3 i_position_offset;
layout (location = 2) in vec3 i_color;

layout (location = 0) out vec3 o_color;

void main() {
    gl_PointSize = 1.0;
    gl_Position = vec4(i_position + i_position_offset, 1.0);

    o_color = i_color;
}
