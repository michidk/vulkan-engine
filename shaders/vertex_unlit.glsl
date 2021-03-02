//# NAME Vertex Unlit
//# DESCRIPTION Renders vertex colors without lighting
//# VERSION 450

//# TYPE VERTEX
layout (location = 0) in vec3 i_position;
layout (location = 1) in vec3 i_color;
layout (location = 4) in mat4 i_modelMatrix;

layout (set = 0, binding = 0) uniform FrameData {
    mat4 view_matrix;
    mat4 projection_matrix;
    vec3 cam_pos;
} u_FrameData;

layout (location = 0) out vec3 o_color;

void main() {
    gl_Position = u_FrameData.projection_matrix * u_FrameData.view_matrix * i_modelMatrix * vec4(i_position, 1.0);
    o_color = i_color;
}

//# TYPE FRAGMENT
layout (location = 0) in vec3 i_color;

layout (location = 0) out vec4 o_color;

void main(){
	o_color = vec4(i_color, 1.0);
}
