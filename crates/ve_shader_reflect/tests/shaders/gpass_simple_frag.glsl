#version 450

layout (location = 0) in V2F {
    vec3 worldNormal;
    vec2 uv;
} v2f;

layout (set=1, binding=0) uniform MaterialData {
    float roughness;
    float metallic;
    vec3 tint;
} u_Material;

layout (set=1, binding=1) uniform sampler2D u_AlbedoTex;

layout (location = 0) out vec4 out_AlbedoRoughness;
layout (location = 1) out vec4 out_NormalMetallic;

void main() {
    vec3 albedo = texture(u_AlbedoTex, v2f.uv).rgb;

    out_AlbedoRoughness = vec4(albedo * u_Material.tint, u_Material.roughness);
    out_NormalMetallic = vec4(v2f.worldNormal, u_Material.metallic);
}
