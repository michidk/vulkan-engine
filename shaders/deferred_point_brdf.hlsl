#include "resolve.hlslh"
#include "brdf.hlslh"

struct Frag {
    float4 color : SV_TARGET0;
};

DEFAULT_VERTEX_SHADER

GBUF0_BINDING SubpassInput in_AlbedoRoughness;
GBUF1_BINDING SubpassInput in_NormalMetallic;
GBUF_DEPTH_BINDING SubpassInput in_Depth;

CAM_BINDING ConstantBuffer<Cam> u_Cam;
LIGHT_BINDING ConstantBuffer<PointLight> u_Light;

Frag frag(V2F fIn) {
    Frag res;

    float4 albedoRoughness = in_AlbedoRoughness.SubpassLoad();
    float4 normalMetallic = in_NormalMetallic.SubpassLoad();
    float depth = in_Depth.SubpassLoad().r;

    float3 albedo = albedoRoughness.rgb;
    float roughness = albedoRoughness.a;
    float3 worldNormal = normalize(normalMetallic.rgb);
    float metallic = normalMetallic.a;

    CALC_WORLD_POS(fIn.uv, depth, u_Cam.invProjMatrix, u_Cam.invViewMatrix);

    float3 dirToCam = normalize(u_Cam.position - worldPos);

    float3 dirToLight = normalize(u_Light.position.xyz - worldPos);
    float d = length(worldPos - u_Light.position);
    float3 irradiance = u_Light.luminousFlux.rgb / (4.0 * PI * d * d);

    float3 radiance = computeRadiance(irradiance, dirToLight, worldNormal, dirToCam, albedo, metallic, roughness);

    res.color = float4(radiance, 1.0);
    return res;
}
