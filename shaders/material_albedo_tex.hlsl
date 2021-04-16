#include "gpass.hlslh"

CAM_BINDING ConstantBuffer<Cam> u_Cam;
TRANSFORM_BINDING ConstantBuffer<Transform> u_Transform;

struct V2F {
    float4 position : SV_POSITION;
    float3 worldNormal;
    float2 uv;
};

struct Frag {
    float4 albedoRoughness : SV_TARGET0;
    float4 normalMetallic : SV_TARGET1;
};

struct MaterialData {
    float metallic;
    float roughness;
};

MATERIAL_BINDING(0) ConstantBuffer<MaterialData> u_Material;
MATERIAL_BINDING(1) Texture2D u_AlbedoTex;

SamplerState s;

V2F vert(Vert vIn) {
    V2F vOut;

    vOut.position = float4(vIn.position, 1.0) * u_Transform.modelMatrix * u_Cam.viewMatrix * u_Cam.projMatrix;
    vOut.worldNormal = (float4(vIn.normal, 0.0) * transpose(u_Transform.invModelMatrix)).xyz;
    vOut.uv = vIn.uv;

    return vOut;
}

Frag frag(V2F fIn) {
    Frag fOut;

    float3 albedo = u_AlbedoTex.Sample(s, fIn.uv).rgb;

    fOut.albedoRoughness = float4(albedo, u_Material.roughness);
    fOut.normalMetallic = float4(fIn.worldNormal, u_Material.metallic);

    return fOut;
}
