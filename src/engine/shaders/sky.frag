#version 100
precision highp float;
uniform mat4 inverse_matrix;
uniform vec3 eye;
varying vec2 uv;
void main(){
    vec4 p=inverse_matrix*vec4(uv,1.0,1.0);
    vec3 ray=normalize(p.xyz/p.w-eye);
    vec3 sky=mix(vec3(0.72,0.80,0.84),vec3(0.35,0.57,0.77),smoothstep(0.0,0.85,ray.y));
    float s=max(dot(ray,normalize(vec3(-0.48,0.82,0.31))),0.0);
    sky+=vec3(1.0,0.77,0.43)*(pow(s,100.0)*0.15+pow(s,1800.0)*0.6);
    gl_FragColor=vec4(sky,1.0);
}
