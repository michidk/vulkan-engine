
struct VertIn {
    float2 position;
    float2 uv;
    float4 color;
};

struct V2F {
    float4 position : SV_POSITION;
    float2 uv;
    float4 color;
};

struct FragOut {
    float4 color : SV_TARGET0;
};

[[vk::binding(0, 0)]] Texture2D u_Texture;
SamplerState s;

struct Matrices {
    float4x4 projMatrix;
};
[[vk::push_constant]] ConstantBuffer<Matrices> u_Matrices;

V2F vert(VertIn vIn) {
    V2F vOut;

    vOut.position = float4(vIn.position, 0.0, 1.0) * u_Matrices.projMatrix;
    vOut.uv = vIn.uv;
    vOut.color = vIn.color;

    return vOut;
}

FragOut frag(V2F fIn) {
    FragOut fOut;

    float4 texColor = u_Texture.Sample(s, fIn.uv);
    float4 finalColor = texColor * fIn.color;

    fOut.color = finalColor;

    return fOut;
}
