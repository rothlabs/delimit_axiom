use const_format::concatcp;
use super::shader_parts::ARROW_HIT;

pub const FACET_CORE: &str = r##"
uniform int max_facet_length;

int get_curve_index(int fi, int nth){
    int ci = -1;
    for(int i = 8; i < max_facet_length-10; i++) {
        if(ci < 0 && nth == int(round(get_geom_texel(fi+i) - 9000000.))){
            ci = fi+i;
        }
    }
    return ci;
}

float[9] get_facet_arrows(int fi, vec2 uv){
    int control_count = int(get_geom_texel(fi + 1));
    int order = int(get_geom_texel(fi + 2));
    int knot_count = control_count + order;
    int knot_i = get_knot_index(fi + 3, knot_count, order, uv.y);
    int nth_control = knot_i - order + 1;
    float[8] basis = get_basis(fi + 3 + knot_i, order, control_count, uv.y);
    float[9] arrows = float[9](0., 0., 0., 0., 0., 0., 0., 0., 0.);
    for(int k = 0; k < order; k++) {
        int ci = get_curve_index(fi, nth_control + k);
        float[6] arrow = get_curve_arrow(ci, uv.x); 
        for(int j = 0; j < 3; j++) {
            arrows[j]   += arrow[j]   * basis[4-order+k];
            arrows[j+3] += arrow[j+3] * basis[4-order+k];
            arrows[j+6] += arrow[j]   * basis[8-order+k];
        }
    }
    return arrows; 
}
"##;


pub const ARROW_IN: &str = r##"
uniform sampler2D point_tex;
uniform sampler2D delta_tex_u;
uniform sampler2D delta_tex_v;
"##;

pub const ARROW_OUT: &str = r##"
layout(location=0) out vec3 point;
layout(location=1) out vec4 delta_u;
layout(location=2) out vec4 delta_v;
void output_arrows(int facet_index, vec2 uv){
    float[9] arrows = get_facet_arrows(facet_index, uv);
    point   = vec3(arrows[0], arrows[1], arrows[2]);
    delta_u = vec4(arrows[3], arrows[4], arrows[5], uv.x);
    delta_v = vec4(arrows[6], arrows[7], arrows[8], uv.y);
}
"##;

pub const ARROW_DUAL: &str = r##"
    ivec2 in_pos0a = out_pos;
    ivec2 in_pos1a = out_pos;
    if(!(out_pos.x < pair_size.x)){
        if(out_pos.x < pair_size.x * 2){ 
            in_pos0a.x = out_pos.x - pair_size.x; 
        }else{
            in_pos0a.x = out_pos.x - pair_size.x * 2; 
        }
        in_pos1a.x = in_pos0a.x;
    }
    if(out_pos.y < pair_size.y){
        in_pos1a.y = out_pos.y + pair_size.y;
    }else{
        in_pos0a.y = out_pos.y - pair_size.y;
    }
    vec4 t0u = texelFetch(delta_tex_u, in_pos0a, 0);
    vec4 t0v = texelFetch(delta_tex_v, in_pos0a, 0);
    vec4 t1u = texelFetch(delta_tex_u, in_pos1a, 0);
    vec4 t1v = texelFetch(delta_tex_v, in_pos1a, 0);
    vec4 uvs = vec4(t0u.a, t0v.a, t1u.a, t1v.a);
    vec3 p0  = texelFetch(point_tex, in_pos0a, 0).xyz;
    vec3 p1  = texelFetch(point_tex, in_pos1a, 0).xyz;
    vec3 d0u = t0u.xyz;
    vec3 d0v = t0v.xyz;
    vec3 d1u = t1u.xyz;
    vec3 d1v = t1v.xyz;
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
    vec4 t0u = vec4(0., 0., 0., 0.);
    vec4 t0v = vec4(0., 0., 0., 0.);
    vec4 t1u = vec4(0., 0., 0., 0.);
    vec4 t1v = vec4(0., 0., 0., 0.);
    if(pick > 1){
        t0u = texelFetch(delta_tex_u, in_pos0c,  0);
        t0v = texelFetch(delta_tex_v, in_pos0c,  0);
        t1u = texelFetch(delta_tex_u, in_pos1c,  0);
        t1v = texelFetch(delta_tex_v, in_pos1c,  0);
    }else if(pick > 0){
        t0u = texelFetch(delta_tex_u, in_pos0b,  0);
        t0v = texelFetch(delta_tex_v, in_pos0b,  0);
        t1u = texelFetch(delta_tex_u, in_pos1a,  0);
        t1v = texelFetch(delta_tex_v, in_pos1a,  0);
    }else{
        t0u = texelFetch(delta_tex_u, in_pos0a,  0);
        t0v = texelFetch(delta_tex_v, in_pos0a,  0);
        t1u = texelFetch(delta_tex_u, in_pos1b,  0);
        t1v = texelFetch(delta_tex_v, in_pos1b,  0);
    }
    vec4 uvs = vec4(t0u.a, t0v.a, t1u.a, t1v.a);
    vec3 d0u = t0u.xyz;
    vec3 d0v = t0v.xyz;
    vec3 d1u = t1u.xyz;
    vec3 d1v = t1v.xyz;
"##;

pub const HONE: &str = r##"
    int facet_index = 0;
    vec2 uv    = vec2(0., 0.);
    vec3 du    = vec3(0., 0., 0.);
    vec3 dv    = vec3(0., 0., 0.);
    vec3 pa    = vec3(0., 0., 0.);
    vec3 pb    = vec3(0., 0., 0.);
    vec3 deltaX = vec3(0., 0., 0.);
    if(out_pos.y < pair_size.y){
        facet_index = texelFetch(pair_tex, in_pos0a, 0).r;
        uv = uvs.rg; du = d0u; dv = d0v;
        pa = p0; pb = p1;
    }else{
        facet_index = texelFetch(pair_tex, in_pos0a, 0).g;
        uv = uvs.ba; du = d1u; dv = d1v;
        pa = p1; pb = p0;
    }
    if(out_pos.x < pair_size.x){
        point    = pa;
        delta_u = vec4(du.x, du.y, du.z, uv.x);
        delta_v = vec4(dv.x, dv.y, dv.z, uv.y);
    }else{
        if(out_pos.x < pair_size.x * 2){
            deltaX = pb - pa;
        }else{
            deltaX = get_facet_convergence_point(uvs.rg, p0, d0u, d0v, uvs.ba, p1, d1u, d1v) - pa;
        }
        uv = get_moved_uv(uv, du, dv, deltaX);
        output_arrows(facet_index, uv);
    }
"##;

pub const FACET_HIT: &str = concatcp!(ARROW_HIT, r##"
vec3 get_facet_convergence_point(vec2 uv0, vec3 p0, vec3 d0u, vec3 d0v, vec2 uv1, vec3 p1, vec3 d1u, vec3 d1v){
    vec3 normal0 = cross(d0u, d0v);
    vec3 normal1 = cross(d1u, d1v);
    vec3 normal_cross = cross(normal0, normal1);
    vec3 cross0 = normalize(cross(normal0, normal_cross));
    vec3 cross1 = normalize(cross(normal1, normal_cross));
    return get_arrow_hit(p0, cross0, p1, cross1);
}
"##);

pub const FACET_EDGE_HIT: &str = concatcp!(ARROW_HIT, r##"
vec3 get_facet_convergence_point(vec2 uv0, vec3 p0, vec3 d0u, vec3 d0v, vec2 uv1, vec3 p1, vec3 d1u, vec3 d1v){
    vec3 normal0 = cross(d0u, d0v);
    vec3 normal1 = cross(d1u, d1v);
    if(uv0.x < 0.0001 || uv0.x > 0.9999){
        normal0 = d0v;
    }else if(uv0.y < 0.0001 || uv0.y > 0.9999){
        normal0 = d0u;
    }
    if(uv1.x < 0.0001 || uv1.x > 0.9999){
        normal1 = d1v;
    }else if(uv1.y < 0.0001 || uv1.y > 0.9999){
        normal1 = d1u;
    }
    vec3 normal_cross = cross(normal0, normal1);
    vec3 cross0 = normalize(cross(normal0, normal_cross));
    vec3 cross1 = normalize(cross(normal1, normal_cross));
    if(uv0.x < 0.0001 || uv0.x > 0.9999){
        cross0 = d0v;
    }else if(uv0.y < 0.0001 || uv0.y > 0.9999){
        cross0 = d0u;
    }
    if(uv1.x < 0.0001 || uv1.x > 0.9999){
        cross1 = d1v;
    }else if(uv1.y < 0.0001 || uv1.y > 0.9999){
        cross1 = d1u;
    }
    return get_arrow_hit(p0, cross0, p1, cross1);
}
"##);

pub const MOVE_UV: &str = r##"
vec2 get_moved_uv(vec2 uv, vec3 du, vec3 dv, vec3 target) {
    if(isnan(target.x) || isnan(target.y) || isnan(target.z) || length(target) < 0.0001){  
        return uv;
    }
    uv = uv + vec2(
        dot(normalize(du), normalize(target)) * length(target) / length(du), 
        dot(normalize(dv), normalize(target)) * length(target) / length(dv)
    );
    uv.x = clamp(uv.x, 0., 1.);
    uv.y = clamp(uv.y, 0., 1.);
    return uv;
}
"##;

pub const MOVE_UV_STOP: &str = r##"
vec2 get_moved_uv(vec2 uv0, vec3 du0, vec3 dv0, vec2 uv1, vec3 du1, vec3 dv1, vec3 target) {
    if(isnan(target.x) || isnan(target.y) || isnan(target.z) || length(target) < 0.0001){  // 0.0001
        return uv0;
    }
    vec2 delta0 = vec2(
        dot(normalize(du0), normalize(target)) * length(target) / length(du0), 
        dot(normalize(dv0), normalize(target)) * length(target) / length(dv0)
    );
    vec2 delta1 = vec2(
        dot(normalize(du1), normalize(target)) * length(target) / length(du1), 
        dot(normalize(dv1), normalize(target)) * length(target) / length(dv1)
    );
    vec2 nd0 = normalize(delta0);
    vec2 nd1 = normalize(delta1);
    if      (nd0.x >  0.01 && uv0.x > 0.9999){ 
        return uv0;
    }else if(nd0.x < -0.01 && uv0.x < 0.0001){ 
        return uv0;
    }else if(nd0.y >  0.01 && uv0.y > 0.9999){ 
        return uv0;
    }else if(nd0.y < -0.01 && uv0.y < 0.0001){ 
        return uv0;
    }
    if      (nd1.x >  0.01 && uv1.x > 0.9999){ 
        return uv0;
    }else if(nd1.x < -0.01 && uv1.x < 0.0001){ 
        return uv0;
    }else if(nd1.y >  0.01 && uv1.y > 0.9999){ 
        return uv0;
    }else if(nd1.y < -0.01 && uv1.y < 0.0001){ 
        return uv0;
    }
    vec2 uv = uv0 + delta0;
    uv.x = clamp(uv.x, 0., 1.);
    uv.y = clamp(uv.y, 0., 1.);
    return uv;
}
"##;


// vec2 get_arrow_middle_2d(vec2 p0, vec2 delta0, vec2 p1, vec2 delta1, vec2 alt) {
//     float dotx = dot(delta0, delta1);
//     if(abs(dotx) > 0.9999) {
//         return alt; 
//     }
//     vec2 delta = p0 - p1;
//     float dot0 = dot(delta, delta0);
//     float dot1 = dot(delta, delta1);
//     float denom = 1. - dotx * dotx;
//     float t = (dotx * dot1 - dot0) / denom;
//     float s = (dot1 - dotx * dot0) / denom;
//     vec2 closest0 = p0 + t * delta0;
//     vec2 closest1 = p1 + s * delta1;
//     return (closest0 + closest1) / 2.;
// }


// pub const MOVE_UV_STICKY: &str = r##"
// vec2 get_moved_uv(vec2 uv_in, vec3 du, vec3 dv, vec3 target) {
//     if(isnan(target.x) || isnan(target.y) || isnan(target.z) || length(target) < 0.0001){  // 0.0001
//         return uv_in;
//     }
//     vec2 uv_delta = vec2(
//         dot(normalize(du), normalize(target)) * length(target) / length(du), 
//         dot(normalize(dv), normalize(target)) * length(target) / length(dv)
//     );
//     vec2 uv = uv_in + uv_delta;
//     if(uv_in.x > 0.99999){ 
//         uv.x = 1.;
//     }else if(uv_in.x < 0.00001){ 
//         uv.x = 0.;
//     }
//     if(uv_in.y > 0.99999){ 
//         uv.y = 1.;
//     }else if(uv_in.y < 0.00001){ 
//         uv.y = 0.;
//     }
//     uv.x = clamp(uv.x, 0., 1.);
//     uv.y = clamp(uv.y, 0., 1.);
//     return uv;
// }
// "##;