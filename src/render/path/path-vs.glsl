in vec4 color;
in vec2 position;

uniform mat4 projection;
uniform mat4 view;

out vec4 f_color;

void main() {
    f_color = color;
    gl_Position = projection * view * vec4(position, 0.0, 1.0);
}
