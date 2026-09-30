#version 100
precision highp float;
uniform sampler2D detail;
uniform sampler2D shadow_map;
uniform vec3 eye;
uniform vec3 sun;
varying vec3 position;
varying vec3 normal;
varying vec3 tangent;
varying vec3 color;
varying vec2 uv;
varying float style;
varying vec4 light_position;
const float PI=3.14159265;
float visibility(vec3 n) {
    vec3 p=light_position.xyz/light_position.w*0.5+0.5;
    if(p.x<0.005 || p.x>0.995 || p.y<0.005 || p.y>0.995 || p.z<0.0 || p.z>1.0) return 1.0;
    float bias=max(0.00022,0.0008*(1.0-max(dot(n,sun),0.0)));
    float value=0.0;
    for(int x=-1;x<=1;x++) for(int y=-1;y<=1;y++) {
        vec3 encoded_depth=texture2D(shadow_map,p.xy+vec2(float(x),float(y))*1.35/2048.0).rgb;
        float depth=dot(encoded_depth,vec3(1.0,1.0/255.0,1.0/65025.0));
        value+=step(p.z-bias,depth);
    }
    // Fade to unshadowed lighting at the coverage boundary.
    float edge=min(min(p.x,1.0-p.x),min(p.y,1.0-p.y));
    return mix(1.0,value/9.0,smoothstep(0.005,0.045,edge));
}
vec3 fresnel(float h,vec3 f0) {return f0+(1.0-f0)*pow(1.0-h,5.0);}
vec3 tone(vec3 x) {return clamp((x*(2.51*x+0.03))/(x*(2.43*x+0.59)+0.14),0.0,1.0);}
void main() {
    float distance_to_eye=length(eye-position);
    vec3 base=pow(max(color,vec3(0.0)),vec3(2.2));
    vec3 n=normalize(normal);
    vec3 t=normalize(tangent-n*dot(n,tangent));
    vec3 b=cross(n,t);
    bool marking=style>1.5 && style<2.5;
    bool paint=style>2.5 && style<3.5;
    bool rubber=style>3.5;
    vec4 tex=texture2D(detail,uv*1.4);
    float relief=paint?0.14:0.42;
    relief*=1.0-smoothstep(12.0,45.0,distance_to_eye);
    if(!marking) {
        n=normalize(n+t*(tex.g*2.0-1.0)*relief+b*(tex.b*2.0-1.0)*relief);
        base*=1.0+(tex.r-0.5)*(paint?0.20:0.65);
    }
    float roughness=paint?0.36:(rubber?0.92:0.78);
    roughness=clamp(roughness+(tex.a-0.5)*0.2,0.22,0.96);
    if(style>0.5 && style<1.5) {
        // Wide, filtered paver joints remain readable without a distant grid shimmer.
        vec2 cell=abs(fract(uv/2.0+0.5)-0.5);
        float joint=smoothstep(0.485,0.497,max(cell.x,cell.y));
        base*=1.0-joint*0.16*(1.0-smoothstep(20.0,90.0,distance_to_eye));
    }
    vec3 v=normalize(eye-position), h=normalize(sun+v);
    float nl=max(dot(n,sun),0.0), nv=max(dot(n,v),0.001);
    float nh=max(dot(n,h),0.0), vh=max(dot(v,h),0.0);
    float a=roughness*roughness, a2=a*a;
    float d=a2/(PI*pow(nh*nh*(a2-1.0)+1.0,2.0)+0.00001);
    float k=pow(roughness+1.0,2.0)/8.0;
    float g=(nv/(nv*(1.0-k)+k))*(nl/(nl*(1.0-k)+k));
    vec3 f=fresnel(vh,vec3(0.04));
    vec3 spec=d*g*f/(4.0*nv*max(nl,0.001));
    float shadow=visibility(normalize(normal));
    vec3 ambient=mix(vec3(0.15,0.17,0.18),vec3(0.48,0.60,0.75),n.y*0.5+0.5);
    vec3 lit=base*ambient+(base*(1.0-f)/PI+spec)*vec3(3.0,2.78,2.45)*nl*shadow;
    if(marking) lit=base*1.15;
    vec3 display=pow(tone(lit),vec3(1.0/2.2));
    float fog=clamp(1.0-exp(-distance_to_eye*0.0028),0.0,0.82);
    gl_FragColor=vec4(mix(display,vec3(0.72,0.80,0.84),fog),1.0);
}
