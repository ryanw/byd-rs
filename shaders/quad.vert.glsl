#version 440

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 vTexCoord;

void main() {
	// Quad always fills the whole screen
	gl_Position = vec4(position, 0.99999, 1.0);
	vTexCoord = position * 0.5 + 0.5;
	vTexCoord.y = 1.0 - vTexCoord.y;
}

