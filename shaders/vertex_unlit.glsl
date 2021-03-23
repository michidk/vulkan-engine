//# NAME Vertex Unlit
//# DESCRIPTION Renders vertex colors without lighting
//# VERSION 450

//# TYPE VERTEX
#include "gpass_defines.glslh"

IN_POSITION in_Position;
IN_COLOR in_Color;

UNIFORM_CAMDATA u_CamData;

UNIFORM_TRANSFORM u_TransformData;

VERTEX_OUT {
    vec3 vertexColor;
} v2f;

void main() {
    gl_Position = u_CamData.projMatrix * u_CamData.viewMatrix * u_TransformData.modelMatrix * vec4(in_Position, 1.0);
    v2f.vertexColor = in_Color;
}

//# TYPE FRAGMENT
#include "gpass_defines.glslh"

FRAGMENT_IN {
    vec3 vertexColor;
} v2f;

OUT_GPASS0 out_VertexColor;

void main(){
	out_VertexColor = vec4(v2f.vertexColor, 0.0);
}
