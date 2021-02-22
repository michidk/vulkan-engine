//# NAME Texture
//# DESCRIPTION Renders a texture
//# VERSION 450

//# TYPE VERTEX
layout (location=0) in vec3 position;
layout (location=1) in vec2 texcoord;
layout (location=2) in mat4 model_matrix;
layout (location=6) in mat4 inverse_model_matrix;

layout (set=0, binding=0) uniform UniformBufferObject {
	mat4 view_matrix;
	mat4 projection_matrix;
} ubo;

layout (location=0) out vec2 uv;

void main() {
    vec4 worldpos = model_matrix*vec4(position,1.0);
    gl_Position = ubo.projection_matrix*ubo.view_matrix*worldpos;
    uv = texcoord;
}

//# TYPE FRAGMENT
layout (location=0) out vec4 theColour;

layout (location=0) in vec2 uv;

layout(set=1,binding=0) uniform sampler2D texturesampler;

void main(){
	theColour=texture(texturesampler,uv);
}
