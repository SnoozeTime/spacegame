in vec2 v_uv;
out vec4 frag;

uniform sampler2D tex;
uniform bool should_blink;
uniform vec4 blink_color;
uniform float time;
uniform float amplitude;

void main() {
    vec4 color = texture(tex, v_uv);
    if (should_blink) {
        color *= blink_color * abs(sin(amplitude*time));
    }
    frag = color;
}