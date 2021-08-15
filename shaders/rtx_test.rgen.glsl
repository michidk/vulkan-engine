#version 460
#extension GL_EXT_ray_tracing : enable

layout(set=0, binding=0) uniform accelerationStructureEXT u_Scene;
layout(set=0, binding=1, rgba16f) uniform image2D u_Image;

layout(set=0, binding=2) uniform Cam {
    mat4 viewMatrix;
    mat4 projMatrix;
    mat4 invViewMatrix;
    mat4 invProjMatrix;
    vec3 position;
} u_Camera;

layout(location=0) rayPayloadEXT vec3 hitColor;

void main() {
    const vec2 pixelCenter = vec2(gl_LaunchIDEXT.xy) + vec2(0.5, 0.5);
    const vec2 clipPos = pixelCenter / vec2(gl_LaunchSizeEXT.xy) * 2.0 - 1.0;

    vec3 origin = u_Camera.position;
    vec3 direction = normalize(u_Camera.invProjMatrix * vec4(clipPos, 1, 1)).xyz;
    direction = (u_Camera.invViewMatrix * vec4(direction, 0.0)).xyz;

    hitColor = vec3(0, 0, 0);
    traceRayEXT(u_Scene, gl_RayFlagsOpaqueEXT, 0xFF, 0, 0, 0, origin, 0.001, direction, 1000.0, 0);

    imageStore(u_Image, ivec2(gl_LaunchIDEXT.xy), vec4(hitColor, 1.0));
}
