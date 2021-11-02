#version 450

layout (location = 0) in vec3 in_Position;
layout (location = 1) in vec3 in_Normal;
layout (location = 2) in vec2 in_UV;

layout (location = 0) out V2F {
    vec3 worldNormal;
    vec2 uv;
} v2f;

layout (set=0, binding=0) uniform FrameData {
    mat4 projMatrix;
    mat4 invProjMatrix;
    mat4 viewMatrix;
    mat4 invViewMatrix;
} u_FrameData;

layout (push_constant) uniform ModelData {
    mat4 modelMatrix;
    mat4 invModelMatrix;
} u_ModelData;

void main() {
    gl_Position = u_FrameData.projMatrix * u_FrameData.viewMatrix * u_ModelData.modelMatrix * vec4(in_Position, 1.0);
    v2f.uv = in_UV;
    v2f.worldNormal = transpose(mat3(u_ModelData.invModelMatrix)) * in_Normal;
}
