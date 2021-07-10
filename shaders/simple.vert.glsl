#version 440

//layout(binding = 0) uniform GlobalUniforms {
//	vec2 uResolution;
//	float uTime;
//} g;

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec2 vTexCoord;
layout(location = 1) out vec4 vColor;

void main() {
	// Quad always fills the whole screen
	gl_Position = vec4(position.xyz, 1.0);
	vTexCoord = position.xy;
	vColor = color;
	//vTexCoord = (position.xy * g.uResolution) / g.uResolution.y;
	//vTexCoord.x += cos(g.uTime);
	//vTexCoord.y += sin(g.uTime);
}
