#version 440

layout(binding = 0)
uniform Camera {
	mat4 view;
	mat4 projection;
	mat4 model;
} camera;


//layout(binding = 0) uniform GlobalUniforms {
//	vec2 uResolution;
//	float uTime;
//} g;

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec2 vTexCoord;
layout(location = 1) out vec4 vColor;

void main() {
	mat4 mvp = camera.projection * camera.view * camera.model;
	// Quad always fills the whole screen
	gl_Position = mvp * vec4(position, 1.0);
	vTexCoord = position.xy;
	vColor = color;
	//vTexCoord = (position.xy * g.uResolution) / g.uResolution.y;
	//vTexCoord.x += cos(g.uTime);
	//vTexCoord.y += sin(g.uTime);
}
