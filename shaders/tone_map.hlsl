#include "pp.hlslh"

struct Frag {
    float4 color : SV_TARGET0;
};

DEFAULT_VERTEX_SHADER

SRC_IMAGE_BINDING Texture2D u_SrcImage;
SamplerState s;

Frag frag(V2F fIn) {
    Frag res;

    float3 src = u_SrcImage.Sample(s, fIn.uv).rgb;

    float3 toneMapped = src / (float3(1.0) + src);
    res.color = float4(toneMapped, 1.0);

    return res;
}
