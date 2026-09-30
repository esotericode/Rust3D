#version 100
precision highp float;
// GLSL ES 1.00's built-in gl_FragCoord can be mediump. Preserve the depth
// from the vertex stage explicitly instead of quantizing it before packing.
varying highp float projected_depth;
void main(){vec3 d=fract(projected_depth*vec3(1.0,255.0,65025.0));d-=d.yzz*vec3(1.0/255.0,1.0/255.0,0.0);gl_FragColor=vec4(d,1.0);}
