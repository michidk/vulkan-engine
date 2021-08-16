#version 460
#extension GL_EXT_ray_tracing : enable
#extension GL_EXT_buffer_reference : enable
#extension GL_EXT_scalar_block_layout : enable

layout(location=0) rayPayloadInEXT vec3 hitColor;

hitAttributeEXT vec3 attribs;

void main() {
    const vec3 baryCoords = vec3(1.0 - attribs.x - attribs.y, attribs.x, attribs.y);
    hitColor = baryCoords;
}
