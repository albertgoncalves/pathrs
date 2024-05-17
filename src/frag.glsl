#version 330 core

layout(location = 0) out vec4 color_frag;

in vec3 color_vert;

void main() {
    color_frag = vec4(color_vert, 1.0);
}
