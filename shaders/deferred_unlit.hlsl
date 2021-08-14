#include "resolve.hlslh"

struct Frag {
    float4 color : SV_TARGET0;
};

CAM_BINDING ConstantBuffer<Cam> u_Cam;
GBUF0_BINDING SubpassInput in_AlbedoRoughness;
GBUF1_BINDING SubpassInput in_NormalMetallic;
GBUF_DEPTH_BINDING SubpassInput in_Depth;

DEFAULT_VERTEX_SHADER

Frag frag(V2F fIn) {
    Frag res;

    float4 albedoRoughness = in_AlbedoRoughness.SubpassLoad();
    float4 normalMetallic = in_NormalMetallic.SubpassLoad();
    float depth = in_Depth.SubpassLoad().r;

    float3 albedo = albedoRoughness.rgb;
    res.color = float4(albedo, 1.0);
    
    return res;
}
