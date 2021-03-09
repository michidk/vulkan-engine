//# NAME Reinhard tone mapping PP
//# DESCRIPTION Post Processing effect that implemented reinhard tone mapping
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
layout (location = 0) in vec2 v2f_UV;

layout (set = 0, binding = 0) uniform sampler2D u_SrcImage;

layout (location = 0) out vec4 out_Color;

void main() {
    vec3 src = texture(u_SrcImage, v2f_UV).rgb;

    vec3 toneMapped = src / (vec3(1.0) + src);
    out_Color = vec4(toneMapped, 1.0);
}
