
#define M_PI 3.1415926535897932384626433832795

const vec2[4] QUAD_POS = vec2[](
vec2(-1., -1.),
vec2( 1., -1.),
vec2( 1.,  1.),
vec2(-1.,  1.)
);

uniform vec3 camera_position;
uniform vec3 center;
uniform vec4 color;
uniform mat4 projection;
uniform mat4 model;
uniform mat4 view;

out vec2 v_uv;
out vec4 v_color;


void main() {
    v_color = color;
    vec2 p = QUAD_POS[gl_VertexID];
    gl_Position = projection * view *  model  * vec4(p, 1.0, 1.0);
    v_uv = p * .5 + .5; // transform the position of the vertex into UV space
}