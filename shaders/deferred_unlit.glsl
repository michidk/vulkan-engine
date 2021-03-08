//# NAME Unlit Deferred
//# DESCRIPTION Unlit Deferred resolve shader
//# VERSION 450

//# TYPE VERTEX
const vec3 g_Vertices[6] = vec3[6](
    vec3(-1.0, -1.0, 0.0),
    vec3(1.0, 1.0, 0.0),
    vec3(-1.0, 1.0, 0.0),

    vec3(-1.0, -1.0, 0.0),
    vec3(1.0, -1.0, 0.0),
    vec3(1.0, 1.0, 0.0)
);

const vec2 g_UVs[6] = vec2[6](
    vec2(-1.0, -1.0),
    vec2(1.0, 1.0),
    vec2(-1.0, 1.0),

    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(1.0, 1.0)
);

layout (location = 0) out vec2 v2f_UV;

void main() {
    gl_Position = vec4(g_Vertices[gl_VertexIndex], 1.0);
    v2f_UV = g_UVs[gl_VertexIndex];
}

//# TYPE FRAGMENT
layout (input_attachment_index = 0, set = 0, binding = 1) uniform subpassInput in_AlbedoRoughness;
layout (input_attachment_index = 1, set = 0, binding = 2) uniform subpassInput in_NormalMetallic;
layout (input_attachment_index = 2, set = 0, binding = 3) uniform subpassInput in_Depth;

layout (set = 0, binding = 0) uniform CamData {
    mat4 viewMatrix;
    mat4 projMatrix;
    mat4 invViewMatrix;
    mat4 invProjMatrix;
    vec3 camPos;
} u_CamData;

layout (location = 0) in vec2 v2f_UV;

layout (location = 0) out vec4 out_Color;

void main() {
    vec4 albedoRoughness = subpassLoad(in_AlbedoRoughness);
    vec4 normalMetallic = subpassLoad(in_NormalMetallic);
    float depth = subpassLoad(in_Depth).r;

    vec3 albedo = albedoRoughness.rgb;
    float roughness = albedoRoughness.a;
    vec3 worldNormal = normalize(normalMetallic.rgb);
    float metallic = normalMetallic.a;

    vec4 clipPos = vec4(v2f_UV.xy, depth, 1.0);
    vec4 viewPos = u_CamData.invProjMatrix * clipPos;
    viewPos /= viewPos.w;
    vec3 worldPos = (u_CamData.invViewMatrix * viewPos).xyz;

    vec3 directionToCamera = normalize(u_CamData.camPos - worldPos);

    out_Color = vec4(albedo, 1.0);
}
