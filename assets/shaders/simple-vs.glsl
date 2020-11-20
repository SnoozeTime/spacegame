in vec2 position;
in vec2 uv;
in vec4 color;

out vec4 v_color;
out vec2 v_uv;

uniform mat4 u_projection;
uniform mat4 u_view;
uniform mat4 u_model;

void main() {
    v_uv = uv;
    v_color = color;
    //gl_Position = vec4(position, 0.0, 1.0);
    gl_Position = u_projection * u_view *  u_model  * vec4(position, 0.0, 1.0);
}