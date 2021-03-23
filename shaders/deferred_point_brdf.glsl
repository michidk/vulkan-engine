//# NAME BRDF Deferred
//# DESCRIPTION Deferred BRDF lighting shader
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

UNIFORM_POINTLIGHT u_LightData;

FRAGMENT_IN {
    vec2 uv;
} v2f;

OUT_COLOR out_Color;

#include "brdf_functions.glslh"

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

    vec3 lightPosition = u_LightData.lightPosition.xyz; // light position
    vec3 luminousFlux = u_LightData.luminousFlux.rgb; // light color in lm, values from https://en.wikipedia.org/wiki/Luminous_flux#Examples

    // light fall-off
    vec3 directionToLight = normalize(lightPosition - worldPos);
    float d = length(worldPos - lightPosition);
    vec3 irradiance = luminousFlux / (4 * PI * d * d);

    vec3 radiance = computeRadiance(irradiance, directionToLight, worldNormal, directionToCamera, albedo, metallic, roughness);
    
    out_Color = vec4(radiance, 1.0);
}
