#version 450

layout (location = 0) in vec3 i_position;
layout (location = 1) in mat4 i_model_matrix;
layout (location = 5) in vec4 i_color;

layout (set = 0, binding = 0) uniform UniformBufferObject {
    mat4 view_matrix;
} ubo;

layout (location = 0) out vec4 o_color;

void main() {
    gl_PointSize = 1.0;
    gl_Position = ubo.view_matrix * i_model_matrix * vec4(i_position, 1.0);

    o_color = i_color;
}
