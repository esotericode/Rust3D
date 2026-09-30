#version 100
precision highp float;
void main(){vec3 d=fract(gl_FragCoord.z*vec3(1.0,255.0,65025.0));d-=d.yzz*vec3(1.0/255.0,1.0/255.0,0.0);gl_FragColor=vec4(d,1.0);}
