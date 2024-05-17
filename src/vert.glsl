#version 330 core

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 translate;
layout(location = 2) in vec2 scale;
layout(location = 3) in vec3 color;

uniform mat4 projection;
uniform mat4 view;

out vec3 color_vert;

void main() {
    gl_Position =
        projection * view * vec4((position * scale) + translate, 0.0, 1.0);
    color_vert = color;
}
