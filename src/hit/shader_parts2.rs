use const_format::concatcp;

pub const ARROW_IN: &str = r##"
uniform sampler2D point_tex;
uniform sampler2D delta_tex;
"##;

pub const ARROW_OUT: &str = r##"
layout(location=0) out vec3 point;
layout(location=1) out vec4 delta;
void output_arrow(int ci, float u){
    float[6] arrow = get_curve_arrow(ci, u);
    point = vec3(arrow[0], arrow[1], arrow[2]);
    delta = vec4(arrow[3], arrow[4], arrow[5], u);
}
"##;

pub const ARROW_PALETTE: &str = r##"
    vec3 p0a = texelFetch(point_tex, in_pos0a, 0).xyz;
    vec3 p0b = texelFetch(point_tex, in_pos0b, 0).xyz;
    vec3 p0c = texelFetch(point_tex, in_pos0c, 0).xyz;
    vec3 p1a = texelFetch(point_tex, in_pos1a, 0).xyz;
    vec3 p1b = texelFetch(point_tex, in_pos1b, 0).xyz;
    vec3 p1c = texelFetch(point_tex, in_pos1c, 0).xyz;
    int pick = 0;
    vec3 p0 = p0a;
    vec3 p1 = p1b;
    if(length(p0b - p1a) < length(p0 - p1)){
        pick = 1;
        p0 = p0b;
        p1 = p1a;
    }
    if(length(p0c - p1c) < length(p0 - p1)){
        pick = 2;
        p0 = p0c;
        p1 = p1c;
    }
    vec4 t0 = vec4(0., 0., 0., 0.);
    vec4 t1 = vec4(0., 0., 0., 0.);
    if(pick > 1){
        t0 = texelFetch(delta_tex, in_pos0c,  0);
        t1 = texelFetch(delta_tex, in_pos1c,  0);
    }else if(pick > 0){
        t0 = texelFetch(delta_tex, in_pos0b,  0);
        t1 = texelFetch(delta_tex, in_pos1a,  0);
    }else{
        t0 = texelFetch(delta_tex, in_pos0a,  0);
        t1 = texelFetch(delta_tex, in_pos1b,  0);
    }
    vec2 us = vec2(t0.a, t1.a);
    vec3 d0u = t0.xyz;
    vec3 d1u = t1.xyz;
"##;

pub const MOVE_U: &str = r##"
float get_moved_u(float u, vec3 du, vec3 delta) {
    if(isnan(delta.x) || isnan(delta.y) || isnan(delta.z) || length(delta) < 0.0001){  
        return u;
    }
    u = u + dot(normalize(du), normalize(delta)) * length(delta) / length(du);
    u = clamp(u, 0., 1.);
    return u;
}
"##;