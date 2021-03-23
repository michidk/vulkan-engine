//# NAME Reinhard tone mapping PP
//# DESCRIPTION Post Processing effect that implemented reinhard tone mapping
//# VERSION 450

//# TYPE VERTEX
#include "pp_defines.glslh"
DEFAULT_VERTEX_SHADER

//# TYPE FRAGMENT
#include "pp_defines.glslh"

FRAGMENT_IN {
    vec2 uv;
} v2f;

UNIFORM_SRC_IMAGE u_SrcImage;

OUT_COLOR out_Color;

void main() {
    vec3 src = texture(u_SrcImage, v2f.uv).rgb;

    vec3 toneMapped = src / (vec3(1.0) + src);
    out_Color = vec4(toneMapped, 1.0);
}
