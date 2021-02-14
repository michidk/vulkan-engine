//# VERSION 450

//# TYPE VERTEX
layout (location = 0) in vec3 i_position;
layout (location = 1) in vec3 i_normal;
layout (location = 2) in mat4 i_modelMatrix;
layout (location = 6) in mat4 i_inverseModelMatrix;
layout (location = 10) in vec4 i_color;
layout (location = 11) in float i_metallic;
layout (location = 12) in float i_roughness;

layout (set = 0, binding = 0) uniform UniformBufferObject {
    mat4 view_matrix;
    mat4 projection_matrix;
} ubo;

layout (location = 0) out vec4 o_color;
layout (location = 1) out vec3 o_normal;
layout (location = 2) out vec4 o_worldPos;
layout (location = 3) out vec3 o_cameraCoordinates;
layout (location = 4) out float o_metallic;
layout (location = 5) out float o_roughness;

void main() {
    o_worldPos = i_modelMatrix * vec4(i_position, 1.0);
    gl_Position = ubo.projection_matrix * ubo.view_matrix * o_worldPos;
    o_color = i_color;
    o_normal = transpose(mat3(i_inverseModelMatrix)) * i_normal;
    o_cameraCoordinates =
        - ubo.view_matrix[3][0] * vec3(ubo.view_matrix[0][0], ubo.view_matrix[1][0], ubo.view_matrix[2][0])
        - ubo.view_matrix[3][1] * vec3(ubo.view_matrix[0][1], ubo.view_matrix[1][1], ubo.view_matrix[2][1])
        - ubo.view_matrix[3][2] * vec3(ubo.view_matrix[0][0], ubo.view_matrix[1][2], ubo.view_matrix[2][2]);
    o_metallic = i_metallic;
    o_roughness = i_roughness;
}


//# TYPE FRAGMENT
layout (location = 0) in vec4 i_color;
layout (location = 1) in vec3 i_normal;
layout (location = 2) in vec4 i_worldpos;
layout (location = 3) in vec3 i_cameraCoordinates;
layout (location = 4) in float i_metallic;
layout (location = 5) in float i_roughness;

layout (location = 0) out vec4 o_color;

const float PI = 3.1415926535897932384626433832795;

readonly layout (set = 1, binding = 0) buffer StorageBufferObject {
    float numDirectional;
    float numPoint;
    vec3 data[];
} sbo;

// what fraction of microsurfaces his this normal?
float distribution(vec3 normal, vec3 halfVector, float roughness) {
    float nDotH = dot(halfVector, normal);
    if (nDotH > 0) {
        float r = roughness * roughness;
        return r / (PI * (1 + nDotH * nDotH * (r - 1)) * (1 + nDotH * nDotH * (r - 1)));
    } else {
        return 0.0;
    }
}

// how visible are the microsurfaces (masking, occlusion)?
float geometry(vec3 light, vec3 normal, vec3 view, float roughness) {
    float nDotRadiance = abs(dot(normal, light));
    float nDotV = abs(dot(normal, view));

    return 0.5 / max(0.01, mix(2 * nDotRadiance * nDotV, nDotRadiance + nDotV, roughness));
}

// Schlicks approximation (approximates r_0 = ((n_1 - n_2)/(n_1 + n_2))^2)
vec3 schlick(vec3 r0, float cosTheta) {
    // we could use pow, but then it do all the float checks - which we don't need
    return r0 + (1 - r0) * (1 - cosTheta) * (1 - cosTheta) * (1 - cosTheta) * (1 - cosTheta) * (1 - cosTheta);
}

vec3 computeRadiance(vec3 irradiance, vec3 lightDirection, vec3 normal, vec3 cameraDirection, vec3 surfaceColor) {

    // utils
    float roughness = i_roughness * i_roughness;
    vec3 halfVector = normalize(0.5 * (cameraDirection + lightDirection));
    float nDotH = max(dot(normal, halfVector), 0);
    float nDotL = max(dot(normal, lightDirection), 0);

    // use 0.03 for non-metallic materials else use the surface color
    vec3 f0 = mix(vec3(0.03), surfaceColor, vec3(i_metallic)); // TODO: move to parameter

    // calculate irradiance by using Fresnel's equations (https://en.wikipedia.org/wiki/Fresnel_equations)
    vec3 irradianceOnSurface = irradiance * nDotL;
    vec3 reflectedIrradiance = schlick(f0, nDotL) * irradianceOnSurface;
    vec3 refractedIrradiance = irradianceOnSurface - reflectedIrradiance;
    vec3 refractedNotAbsorbedIrradiance = refractedIrradiance * (1 - i_metallic);

    vec3 F = schlick(f0, nDotH); // Fresnel coefficient (What part of the incoming light is reflected?)
    vec3 relevantReflection = reflectedIrradiance * F * distribution(normal, halfVector, roughness) * geometry(lightDirection, normal, cameraDirection, roughness);

    return refractedNotAbsorbedIrradiance * surfaceColor / PI + relevantReflection;
}

void main(){
    vec3 radiance = vec3(0);
    vec3 i_normal = normalize(i_normal);
    vec3 directionToCamera = normalize(i_cameraCoordinates - i_worldpos.xyz);

    int number_directional = int(sbo.numDirectional);
    int number_point = int(sbo.numPoint);

    // directional lights
    for (int i = 0; i < number_directional; i++) {
        vec3 directionToLight = sbo.data[2 * i]; // direction to light
        vec3 irradiance = sbo.data[2 * i + 1]; // light color in lx (= lm/m^2), values from https://en.wikipedia.org/wiki/Lux#Illuminance

        radiance += computeRadiance(irradiance, normalize(directionToLight), i_normal, directionToCamera, i_color.xyz);
    }

    // point lights
    for (int i = 0; i < number_point; i++) {
        vec3 lightPosition = sbo.data[2 * i + 2 * number_directional]; // light position
        vec3 luminousFlux = sbo.data[2 * i + 1 + 2 * number_directional]; // light color in lm, values from https://en.wikipedia.org/wiki/Luminous_flux#Examples

        // light fall-off
        vec3 directionToLight = normalize(lightPosition - i_worldpos.xyz);
        float d = length(i_worldpos.xyz - lightPosition);
        vec3 irradiance = luminousFlux / (4 * PI * d * d);

        radiance += computeRadiance(irradiance, directionToLight, i_normal, directionToCamera, i_color.xyz);
    };

    radiance = radiance / (1 + radiance); // Reinhard tone mapping

    o_color = vec4(radiance, 1.0);
}
