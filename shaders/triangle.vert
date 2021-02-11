#version 450

layout (location = 0) in vec3 i_position;
layout (location = 1) in vec3 i_normal;
layout (location = 2) in mat4 i_model_matrix;
layout (location = 6) in mat4 i_inverse_model_matrix;
layout (location = 10) in vec4 i_color;

layout (set = 0, binding = 0) uniform UniformBufferObject {
    mat4 view_matrix;
    mat4 projection_matrix;
} ubo;

layout (location = 0) out vec4 o_color;
layout (location = 1) out vec3 o_normal;

void main() {
    gl_PointSize = 1.0;
    gl_Position = ubo.projection_matrix * ubo.view_matrix * i_model_matrix * vec4(i_position, 1.0);

    o_color = vec4(i_position, 1.0);
    //o_normal = vec3(transpose(inverse(i_model_matrix)) * vec4(i_normal, 0.0));
    o_normal = transpose(mat3(i_inverse_model_matrix)) * i_normal;
}
