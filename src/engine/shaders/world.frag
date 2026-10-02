#version 100
precision highp float;
uniform sampler2D detail;
// Packed depth needs full sampler precision; lowp/mediump causes depth bands.
uniform highp sampler2D shadow_map;
uniform mat4 light_matrix;
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
    // Compare each tap against the receiver plane at that texel's centre.
    // A constant depth comparison makes large, sloped surfaces shadow themselves.
    vec3 rx=vec3(light_matrix[0][0],light_matrix[1][0],light_matrix[2][0]);
    vec3 ry=vec3(light_matrix[0][1],light_matrix[1][1],light_matrix[2][1]);
    vec3 rz=vec3(light_matrix[0][2],light_matrix[1][2],light_matrix[2][2]);
    vec3 plane=vec3(dot(rx,n)/dot(rx,rx),dot(ry,n)/dot(ry,ry),dot(rz,n)/dot(rz,rz));
    vec2 slope=abs(plane.z)>0.01?clamp(-plane.xy/plane.z,vec2(-4.0),vec2(4.0)):vec2(0.0);
    float value=0.0;
    for(int x=-1;x<=1;x++) for(int y=-1;y<=1;y++) {
        vec2 tap=(floor((p.xy+vec2(float(x),float(y))*1.35/2048.0)*2048.0)+0.5)/2048.0;
        vec3 encoded_depth=texture2D(shadow_map,tap).rgb;
        float depth=dot(encoded_depth,vec3(1.0,1.0/255.0,1.0/65025.0));
        value+=step(p.z+dot(slope,tap-p.xy)-bias,depth);
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
    bool rubber=style>3.5 && style<4.5;
    bool terrain=style>4.5;
    // Terrain spans kilometres, so its detail tile is three times coarser.
    vec4 tex=texture2D(detail,uv*(terrain?0.45:1.4));
    float relief=paint?0.14:(terrain?0.32:0.42);
    relief*=1.0-smoothstep(10.0,terrain?32.0:45.0,distance_to_eye);
    // Fine albedo grain fades with distance like the relief. Minified at
    // grazing angles, a repeating tile otherwise reads as regular ripples.
    float grain=1.0-smoothstep(15.0,80.0,distance_to_eye);
    if(!marking) {
        n=normalize(n+t*(tex.g*2.0-1.0)*relief+b*(tex.b*2.0-1.0)*relief);
        base*=1.0+(tex.r-0.5)*(paint?0.20:0.65)*grain;
    }
    if(terrain) {
        // Broad, non-repeating-looking patches keep distant ground from
        // flattening once the fine grain has faded.
        float patches=texture2D(detail,uv*0.019+vec2(0.37,0.11)).r*0.6
            +texture2D(detail,uv*0.0061+vec2(0.71,0.53)).r*0.4;
        base*=mix(vec3(0.84,0.86,0.80),vec3(1.12,1.10,1.02),smoothstep(0.38,0.62,patches));
    }
    float roughness=paint?0.36:((rubber || terrain)?0.92:0.78);
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
    float fog=clamp(1.0-exp(-distance_to_eye*0.00085),0.0,0.82);
    gl_FragColor=vec4(mix(display,vec3(0.72,0.80,0.84),fog),1.0);
}
