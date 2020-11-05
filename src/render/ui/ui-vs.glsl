in vec4 color;
in vec2 position;

out vec4 f_color;

void main() {
    f_color = color;
    gl_Position = vec4(position, 0.0, 1.0);
}
