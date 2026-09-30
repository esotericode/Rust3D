#version 100
attribute vec3 in_pos;
uniform mat4 matrix;
varying highp float projected_depth;
void main(){
    gl_Position=matrix*vec4(in_pos,1.0);
    // The sun projection is orthographic, so this interpolates linearly.
    projected_depth=gl_Position.z*0.5+0.5;
}
