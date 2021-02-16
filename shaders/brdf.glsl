//# NAME BRDF
//# DESCRIPTION Renders an object using the Cook-Torrance BRDF
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
layout (location = 0) in vec4 i_color; // TODO: rename to albedo
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


// normal distribution function: Trowbridge-Reitz GGX
float distributionGGX(vec3 normal, vec3 halfVector, float roughness) {
    float a = roughness * roughness; // rougness apparently looks more "correct", when beeing squared (according to Disney)
    float a2 = a * a;
    float nDotH  = max(dot(normal, halfVector), 0.0);
    float nDotH2 = nDotH * nDotH;

    float denom = (nDotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return a2 / denom;
}

// geometry function: Schlick-GGX
float geometrySchlickGGX(float vec, float roughness) {
    float r = roughness + 1.0;
    float k = (r * r) / 8.0;

    float denom = vec * (1.0 - k) + k;

    return vec / denom;
}

// approximate geometry: account for view dir and light dir: Smith's method
float geometrySmith(float nDotV, float nDotL, float roughness) {
    return geometrySchlickGGX(nDotL, roughness) * geometrySchlickGGX(nDotV, roughness);
}

// Schlicks approximation (approximates r_0 = ((n_1 - n_2)/(n_1 + n_2))^2)
vec3 schlick(vec3 r0, float cosTheta) {
    // we could use pow, but then it do all the float checks - which we don't need
    return r0 + (1.0 - r0) * (1.0 - cosTheta) * (1.0 - cosTheta) * (1.0 - cosTheta) * (1.0 - cosTheta) * (1.0 - cosTheta);
}

vec3 computeRadiance(vec3 irradiance, vec3 lightDirection, vec3 normal, vec3 cameraDirection, vec3 surfaceColor) {

    // utils
    vec3 halfVector = normalize(cameraDirection + lightDirection);
    float nDotH = max(dot(normal, halfVector), 0.0);
    float nDotL = max(dot(normal, lightDirection), 0.0);
    float hDotV = max(dot(halfVector, cameraDirection), 0.0);
    float nDotV = max(dot(normal, cameraDirection), 0.0);

    vec3 f0 = mix(vec3(0.04), surfaceColor, vec3(i_metallic)); // base relectivity: use 0.04 for non-metallic/dialectic materials else use the surface color
    // vec3 f0 = vec3(1, 0.86, 0.57);
    vec3 f = schlick(f0, hDotV);

    float ndf = distributionGGX(normal, halfVector, i_roughness);
    float geometry = geometrySmith(nDotV, nDotL, i_roughness);

    // Cook-Torrance BRDF
    vec3 numerator = ndf * geometry * f;
    float denominator = 4.0 * nDotV * nDotL;
    vec3 specular = numerator / max(denominator, 0.001);

    vec3 kS = f; // energy of light that gets reflected
    vec3 kD = vec3(1.0) - kS; // remaining light that gets refracted
    kD *= 1.0 - i_metallic; // metalls don't refract, so set it to 0 if it's a metal

    return (kD * i_color.xyz / PI + specular) * irradiance * nDotL;
}

void main() {
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

    radiance = radiance / (vec3(1.0) + radiance); // Reinhard tone mapping
    // radiance.xyz = pow(radiance.xyz, vec3(1.0/2.2)); // gamma correction

    o_color = vec4(radiance, 1.0);
}
