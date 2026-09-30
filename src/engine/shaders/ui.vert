#version 100
attribute vec3 in_pos;
attribute vec3 in_color;
uniform mat4 matrix;
varying vec3 color;
void main(){gl_Position=matrix*vec4(in_pos,1.0);color=in_color;}
