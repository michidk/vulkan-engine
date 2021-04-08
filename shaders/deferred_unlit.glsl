//# NAME Unlit Deferred
//# DESCRIPTION Unlit Deferred resolve shader
//# VERSION 450

//# TYPE VERTEX
#include "resolve_defines.glslh"
DEFAULT_VERTEX_SHADER

//# TYPE FRAGMENT
#include "resolve_defines.glslh"

IN_GPASS0 in_AlbedoRoughness;
IN_GPASS1 in_NormalMetallic;
IN_DEPTH in_Depth;

UNIFORM_CAMDATA u_CamData;

FRAGMENT_IN {
    vec2 uv;
} v2f;

OUT_COLOR out_Color;

void main() {
    vec4 albedoRoughness = subpassLoad(in_AlbedoRoughness);
    vec4 normalMetallic = subpassLoad(in_NormalMetallic);
    float depth = subpassLoad(in_Depth).r;

    vec3 albedo = albedoRoughness.rgb;
    float roughness = albedoRoughness.a;
    vec3 worldNormal = normalize(normalMetallic.rgb);
    float metallic = normalMetallic.a;

    CALC_WORLD_POS(v2f.uv, depth, u_CamData.invProjMatrix, u_CamData.invViewMatrix);

    vec3 directionToCamera = normalize(u_CamData.camPos - worldPos);

    out_Color = vec4(albedo, 1.0);
}
