//# NAME Solid Color Material
//# DESCRIPTION Deferred GPass Shader for a solid single-color material
//# VERSION 450

//# TYPE VERTEX
layout (location = 0) in vec3 in_Position;
//layout (location = 1) in vec3 in_Color;
layout (location = 2) in vec3 in_Normal;
//layout (location = 3) in vec2 in_UV;

layout (set = 0, binding = 0) uniform CamData {
    mat4 viewMatrix;
    mat4 projMatrix;
    mat4 invViewMatrix;
    mat4 invProjMatrix;
    vec3 camPos;
} u_CamData;

layout (push_constant) uniform TransformData {
    mat4 modelMatrix;
    mat4 invModelMatrix;
} u_TransformData;

layout (location = 0) out vec3 v2f_WorldNormal;

void main() {
    gl_Position = u_CamData.projMatrix * u_CamData.viewMatrix * u_TransformData.modelMatrix * vec4(in_Position, 1.0);
    v2f_WorldNormal = (transpose(u_TransformData.invModelMatrix) * vec4(in_Normal, 0.0)).xyz;
}

//# TYPE FRAGMENT
layout (location = 0) in vec3 v2f_WorldNormal;

layout (location = 0) out vec4 out_AlbedoRoughness;
layout (location = 1) out vec4 out_NormalMetallic;

layout (set = 1, binding = 0) uniform MaterialData {
    vec4 albedo;
    float metallic;
    float roughness;
} u_MaterialData;

void main() {
    out_AlbedoRoughness = vec4(u_MaterialData.albedo.xyz, u_MaterialData.roughness);
    out_NormalMetallic = vec4(v2f_WorldNormal, u_MaterialData.metallic);
}
