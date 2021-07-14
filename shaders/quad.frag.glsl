#version 440

layout(binding = 0) uniform texture2D uTexture;
layout(binding = 1) uniform sampler uSampler;

layout(location=0) in vec2 vTexCoord;
layout(location=0) out vec4 fColor;

void main() {
	vec4 color = texture(sampler2D(uTexture, uSampler), vTexCoord);
	fColor = pow(color, vec4(2.2));
}
