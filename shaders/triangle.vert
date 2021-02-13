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
layout (location = 2) out vec4 o_worldpos;
layout (location = 3) out vec3 o_camera_coordinates;

void main() {
    gl_PointSize = 1.0;
    o_worldpos = i_model_matrix * vec4(i_position, 1.0);
    gl_Position = ubo.projection_matrix * ubo.view_matrix * o_worldpos;
    o_color = i_color;
    o_normal = transpose(mat3(i_inverse_model_matrix)) * i_normal;
    o_camera_coordinates =
        - ubo.view_matrix[3][0] * vec3(ubo.view_matrix[0][0], ubo.view_matrix[1][0], ubo.view_matrix[2][0])
        - ubo.view_matrix[3][1] * vec3(ubo.view_matrix[0][1], ubo.view_matrix[1][1], ubo.view_matrix[2][1])
        - ubo.view_matrix[3][2] * vec3(ubo.view_matrix[0][0], ubo.view_matrix[1][2], ubo.view_matrix[2][2]);
}
