#version 330 core

layout(location = 0) out vec4 color_frag;

in vec4 color_vert;

void main() {
    color_frag = color_vert;
}
