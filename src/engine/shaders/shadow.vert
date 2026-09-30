#version 100
attribute vec3 in_pos;
uniform mat4 matrix;
void main(){gl_Position=matrix*vec4(in_pos,1.0);}
