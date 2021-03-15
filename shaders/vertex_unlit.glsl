//# NAME Vertex Unlit
//# DESCRIPTION Renders vertex colors without lighting
//# VERSION 450

//# TYPE VERTEX
layout (location = 0) in vec3 in_Position;
layout (location = 1) in vec3 in_Color;
//layout (location = 2) in vec3 in_Normal;
//layout (location = 3) in vec2 in_UV;

layout (set = 0, binding = 0) uniform FrameData {
    mat4 viewMatrix;
    mat4 projMatrix;
    mat4 invViewMatrix;
    mat4 invProjMatrix;
    vec3 camPos;
} u_FrameData;

layout (push_constant) uniform TransformData {
    mat4 modelMatrix;
    mat4 invModelMatrix;
} u_TransformData;

layout (location = 0) out vec3 v2f_VertexColor;

void main() {
    gl_Position = u_FrameData.projMatrix * u_FrameData.viewMatrix * u_TransformData.modelMatrix * vec4(in_Position, 1.0);
    v2f_VertexColor = in_Color;
}

//# TYPE FRAGMENT
layout (location = 0) in vec3 v2f_VertexColor;

layout (location = 0) out vec4 out_GBuffer0;
layout (location = 1) out vec4 out_GBuffer1;

void main(){
	out_GBuffer0 = vec4(v2f_VertexColor, 0.0);
}
