#version 440

layout(location=0) in vec2 vTexCoord;
layout(location=1) in vec4 vColor;

layout(location=0) out vec4 fColor;

void main() {
	fColor = pow(vColor, vec4(2.0));
}
