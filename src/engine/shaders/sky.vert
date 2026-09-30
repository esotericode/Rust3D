#version 100
attribute vec2 pos;
varying vec2 uv;
void main(){gl_Position=vec4(pos,1.0,1.0);uv=pos;}
