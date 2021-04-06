//# NAME BRDF Deferred
//# DESCRIPTION Deferred BRDF lighting shader
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
layout (input_attachment_index = 0, set = 0, binding = 1) uniform subpassInput in_AlbedoRoughness;
layout (input_attachment_index = 1, set = 0, binding = 2) uniform subpassInput in_NormalMetallic;
layout (input_attachment_index = 2, set = 0, binding = 3) uniform subpassInput in_Depth;

layout (set = 0, binding = 0) uniform CamData {
    mat4 viewMatrix;
    mat4 projMatrix;
    mat4 invViewMatrix;
    mat4 invProjMatrix;
    vec3 camPos;
} u_CamData;

layout (push_constant) uniform LightData {
    vec4 lightPosition;
    vec4 luminousFlux;
} u_LightData;

layout (location = 0) in vec2 v2f_UV;

layout (location = 0) out vec4 out_Color;

const float PI = 3.1415926535897932384626433832795;

// Schlicks approximation (approximates r_0 = ((n_1 - n_2)/(n_1 + n_2))^2)
vec3 schlick(vec3 r0, float cosTheta) {
    // we could use pow, but then it do all the float checks - which we don't need
    return r0 + (1.0 - r0) * (1.0 - cosTheta) * (1.0 - cosTheta) * (1.0 - cosTheta) * (1.0 - cosTheta) * (1.0 - cosTheta);
}

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

vec3 computeRadiance(vec3 irradiance, vec3 lightDirection, vec3 normal, vec3 cameraDirection, vec3 surfaceColor, float metallic, float roughness) {
    // utils
    vec3 halfVector = normalize(cameraDirection + lightDirection);
    float nDotH = max(dot(normal, halfVector), 0.0);
    float nDotL = max(dot(normal, lightDirection), 0.0);
    float hDotV = max(dot(halfVector, cameraDirection), 0.0);
    float nDotV = max(dot(normal, cameraDirection), 0.0);

    vec3 f0 = mix(vec3(0.04), surfaceColor, vec3(metallic)); // base relectivity: use 0.04 for non-metallic/dialectic materials else use the surface color
    vec3 f = schlick(f0, hDotV);

    float ndf = distributionGGX(normal, halfVector, roughness);
    float geometry = geometrySmith(nDotV, nDotL, roughness);

    // Cook-Torrance BRDF
    vec3 numerator = ndf * geometry * f;
    float denominator = 4.0 * nDotV * nDotL;
    vec3 specular = numerator / max(denominator, 0.001);

    vec3 kS = f; // energy of light that gets reflected
    vec3 kD = vec3(1.0) - kS; // remaining light that gets refracted
    kD *= 1.0 - metallic; // metalls don't refract, so set it to 0 if it's a metal

    return (kD * surfaceColor / PI + specular) * irradiance * nDotL;
}

void main() {
    vec4 albedoRoughness = subpassLoad(in_AlbedoRoughness);
    vec4 normalMetallic = subpassLoad(in_NormalMetallic);
    float depth = subpassLoad(in_Depth).r;

    vec3 albedo = albedoRoughness.rgb;
    float roughness = albedoRoughness.a;
    vec3 worldNormal = normalize(normalMetallic.rgb);
    float metallic = normalMetallic.a;

    vec4 clipPos = vec4(v2f_UV.xy, depth, 1.0);
    vec4 viewPos = u_CamData.invProjMatrix * clipPos;
    viewPos /= viewPos.w;
    vec3 worldPos = (u_CamData.invViewMatrix * viewPos).xyz;

    vec3 directionToCamera = normalize(u_CamData.camPos - worldPos);

    vec3 lightPosition = u_LightData.lightPosition.xyz; // light position
    vec3 luminousFlux = u_LightData.luminousFlux.rgb; // light color in lm, values from https://en.wikipedia.org/wiki/Luminous_flux#Examples

    // light fall-off
    vec3 directionToLight = normalize(lightPosition - worldPos);
    float d = length(worldPos - lightPosition);
    vec3 irradiance = luminousFlux / (4 * PI * d * d);

    vec3 radiance = computeRadiance(irradiance, directionToLight, worldNormal, directionToCamera, albedo, metallic, roughness);
    //radiance = radiance / (vec3(1.0) + radiance); // Reinhard tone mapping

    out_Color = vec4(radiance, 1.0);
}
