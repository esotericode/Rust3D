#version 100
attribute vec3 in_pos;
attribute vec3 in_normal;
attribute vec3 in_color;
attribute vec2 in_uv;
attribute vec3 in_tangent;
attribute float in_style;
uniform mat4 matrix;
uniform mat4 light_matrix;
varying vec3 position;
varying vec3 normal;
varying vec3 tangent;
varying vec3 color;
varying vec2 uv;
varying float style;
varying vec4 light_position;
void main() {
    gl_Position=matrix*vec4(in_pos,1.0);
    position=in_pos; normal=in_normal; tangent=in_tangent;
    color=in_color; uv=in_uv; style=in_style;
    light_position=light_matrix*vec4(in_pos,1.0);
}
