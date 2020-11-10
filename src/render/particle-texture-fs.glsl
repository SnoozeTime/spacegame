in vec2 v_uv;
in vec4 v_color;

out vec4 frag;

uniform sampler2D tex;

void main() {
    vec4 tex_color = texture(tex, v_uv);
    frag = tex_color * v_color;
}