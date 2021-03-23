//# NAME Solid Color Material
//# DESCRIPTION Deferred GPass Shader for a solid single-color material
//# VERSION 450

//# TYPE VERTEX
#include "gpass_defines.glslh"

IN_POSITION in_Position;
IN_NORMAL in_Normal;

UNIFORM_CAMDATA u_CamData;

UNIFORM_TRANSFORM u_TransformData;

VERTEX_OUT {
    vec3 worldNormal;
} v2f;

void main() {
    gl_Position = u_CamData.projMatrix * u_CamData.viewMatrix * u_TransformData.modelMatrix * vec4(in_Position, 1.0);
    v2f.worldNormal = (transpose(u_TransformData.invModelMatrix) * vec4(in_Normal, 0.0)).xyz;
}

//# TYPE FRAGMENT
#include "gpass_defines.glslh"

FRAGMENT_IN {
    vec3 worldNormal;
} v2f;

OUT_GPASS0 out_AlbedoRoughness;
OUT_GPASS1 out_NormalMetallic;

MAT_UNIFORM(0) {
    vec4 albedo;
    float metallic;
    float roughness;
} u_MaterialData;

void main() {
    out_AlbedoRoughness = vec4(u_MaterialData.albedo.xyz, u_MaterialData.roughness);
    out_NormalMetallic = vec4(v2f.worldNormal, u_MaterialData.metallic);
}
