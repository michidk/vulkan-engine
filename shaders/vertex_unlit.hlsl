#include "gpass.hlslh"

struct V2F {
    float4 position : SV_POSITION;
    float3 vertexColor;
};

struct Frag {
    float4 vertexColor : SV_TARGET0;
};

CAM_BINDING ConstantBuffer<Cam> u_Cam;
TRANSFORM_BINDING ConstantBuffer<Transform> u_Transform;

V2F vert(Vert vIn) {
    V2F res;

    res.position = float4(vIn.position, 1.0) * u_Transform.modelMatrix * u_Cam.viewMatrix * u_Cam.projMatrix;
    res.vertexColor = vIn.color;

    return res;
}

Frag frag(V2F fIn) {
    Frag res;

    res.vertexColor = float4(fIn.vertexColor, 0.0);

    return res;
}
