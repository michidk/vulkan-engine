#version 450

layout (location = 0) in vec4 i_color;
layout (location = 1) in vec3 i_normal;

layout (location = 0) out vec4 o_color;

void main(){
    vec3 direction_to_light = normalize(vec3(-1, -1, 0));
    o_color = 0.5 * (1 + max(dot(i_normal, direction_to_light), 0)) * i_color;
}
