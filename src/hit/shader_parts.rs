pub const HEADER: &str = r##"#version 300 es
precision highp float;
precision highp sampler2D;
precision highp isampler2D;
uniform isampler2D pair_tex;
"##; 

pub const CORE_PARTS: &str = r##"
ivec2 pair_size = textureSize(pair_tex, 0);
ivec2 out_pos = ivec2(gl_FragCoord.x, gl_FragCoord.y);
"##; 

pub const FACET_PARTS: &str = r##"
    facet_tex_width = textureSize(facet_tex, 0).x;
"##;

pub const FACET_CORE: &str = r##"
uniform sampler2D facet_tex;
uniform int max_facet_length;
uniform int max_knot_count;

int facet_tex_width = 0;

float get_facet_texel(int index) {
    int y = index / facet_tex_width;
    int x = index % facet_tex_width;
    return texelFetch(facet_tex, ivec2(x, y), 0).r;
}

int get_curve_index(int index, int nth){
    int idx = -1;
    for(int i = 8; i < max_facet_length-10; i++) {
        if(idx < 0 && nth == int(round(get_facet_texel(index+i) - 9000000.))){
            idx = index+i;
        }
    }
    return idx;
}

int get_knot_index(int idx, int knot_count, int order, float u){
    for(int i = 0; i < max_knot_count-1; i++) { 
        if(i < knot_count && u >= get_facet_texel(idx + i) && u < get_facet_texel(idx + i + 1)) { 
            return i; // knot_i = i;
        }
    }
    return knot_count - order - 1;
}

float[8] get_basis(int ki, int order, int control_len, float u){
    float k0  = get_facet_texel(ki);
    float k1  = get_facet_texel(ki + 1);
    float k1u  = k1 - u;
    float uk0  = u - k0;
    float k0k1 = k0 - k1;
    float k1k0 = k1 - k0;
    float k1u_d_k1k0 = k1u / k1k0;
    float uk0_d_k1k0 = uk0 / k1k0;
    if(order > 2){ // quadratic
        float r1 = get_facet_texel(ki - 1);
        float k2 = get_facet_texel(ki + 2);
        float w0 = get_facet_texel(ki + control_len + 1);
        float w1 = get_facet_texel(ki + control_len + 2);
        float w2 = get_facet_texel(ki + control_len + 3);
            // origin parts:
        float k0u = k0 - u;
        float k2u = k2 - u;
        float ur1 = u - r1;
        float r1k2 = r1 - k2;
        float k0k2 = k0 - k2;
        float k1r1 = k1 - r1;
        float k2k0 = k2 - k0;
        float w0xk1u = w0 * k1u;
        float w2xuk0 = w2 * uk0;
        float p0 = w0xk1u * k1u_d_k1k0 / k1r1;
        float p1 = (k1u_d_k1k0 * ur1/k1r1 + uk0_d_k1k0 * k2u/k2k0) * w1;
        float p2 = w2xuk0 * uk0_d_k1k0 / k2k0;
        float sum = p0 + p1 + p2;
            // derivative parts:
        float a0 = 2. * k0k1 * k0k2 * k1r1;
        float n0 = a0 * w0xk1u * (w1 * (u-k2) - w2xuk0);
        float n1 = a0 * w1 * (w0 * k1u * k2u - w2xuk0 * ur1);
        float n2 = a0 * w2xuk0 * (w0 * k1u + w1 * ur1);
        float uxu = u * u;
        float k2xr1 = k2 * r1;
        float ux2 = u * 2.;
        float a1 = - w0xk1u * k0k2 * k1u + w1 * (k0 * (k1 * r1k2 + k2xr1 - r1 * ux2 + uxu) - k1*(k2xr1 - k2 * ux2 + uxu) + uxu * r1k2);
        float d0 = a1 + w2xuk0 * uk0 * k1r1;
        float d1 = a1 + w2 * k0u * k0u * k1r1;
        return float[8](0., p0/sum, p1/sum, p2/sum, 0., n0/d0/d0, n1/d0/d0, n2/d1/d1);
    } else { // linear
        return float[8](0., 0., k1u_d_k1k0, uk0_d_k1k0, 0., 0., 1./k0k1, 1./k1k0);
    }
}

float[6] get_curve_arrow(int idx, int nth, float u){
    int ci = get_curve_index(idx, nth);
    int control_count = int(get_facet_texel(ci + 1));
    int order = int(get_facet_texel(ci + 2));
    float min = get_facet_texel(ci + 3);
    float max = get_facet_texel(ci + 4);
    int knot_count = control_count + order;
    u = min*(1.-u) + max*u;
    float velocity_scale = max - min;
    int knot_i = get_knot_index(ci + 5, knot_count, order, u);
    int control_start = ci + 5 + knot_count + control_count + (knot_i-order+1)*3;
    float[8] basis = get_basis(ci + 5 + knot_i, order, control_count, u);
    float[6] arrow = float[6](0., 0., 0., 0., 0., 0.);
    for(int k = 0; k < order; k++) {
        for(int j = 0; j < 3; j++) {
            float control_component = get_facet_texel(control_start + k*3 + j);
            arrow[j]   += control_component * basis[4-order+k];
            arrow[j+3] += control_component * basis[8-order+k] * velocity_scale;
        }
    }
    return arrow; 
}

float[9] get_facet_arrows(int fi, vec2 uv){
    int control_count = int(get_facet_texel(fi + 1));
    int order = int(get_facet_texel(fi + 2));
    int knot_count = control_count + order;
    int knot_i = get_knot_index(fi + 3, knot_count, order, uv.y);
    int nth_control = knot_i - order + 1;
    float[8] basis = get_basis(fi + 3 + knot_i, order, control_count, uv.y);
    float[9] arrows = float[9](0., 0., 0., 0., 0., 0., 0., 0., 0.);
    for(int k = 0; k < order; k++) {
        float[6] arrow = get_curve_arrow(fi, nth_control + k, uv.x); 
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
uniform sampler2D origin_tex;
uniform sampler2D vector_tex_u;
uniform sampler2D vector_tex_v;
"##;

pub const ARROW_OUT: &str = r##"
layout(location=0) out vec3 origin;
layout(location=1) out vec4 vector_u;
layout(location=2) out vec4 vector_v;
void output_arrows(int facet_index, vec2 uv){
    float[9] arrows = get_facet_arrows(facet_index, uv);
    origin   = vec3(arrows[0], arrows[1], arrows[2]);
    vector_u = vec4(arrows[3], arrows[4], arrows[5], uv.x);
    vector_v = vec4(arrows[6], arrows[7], arrows[8], uv.y);
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
    vec4 t0u = texelFetch(vector_tex_u, in_pos0a, 0);
    vec4 t0v = texelFetch(vector_tex_v, in_pos0a, 0);
    vec4 t1u = texelFetch(vector_tex_u, in_pos1a, 0);
    vec4 t1v = texelFetch(vector_tex_v, in_pos1a, 0);
    vec4 uvs = vec4(t0u.a, t0v.a, t1u.a, t1v.a);
    vec3 p0  = texelFetch(origin_tex, in_pos0a, 0).xyz;
    vec3 p1  = texelFetch(origin_tex, in_pos1a, 0).xyz;
    vec3 d0u = t0u.xyz;
    vec3 d0v = t0v.xyz;
    vec3 d1u = t1u.xyz;
    vec3 d1v = t1v.xyz;
"##;

pub const ARROW_IN_POS: &str = r##"
    ivec2 in_pos0a = out_pos;
    ivec2 in_pos0b = out_pos;
    ivec2 in_pos0c = out_pos;
    ivec2 in_pos1a = out_pos;
    ivec2 in_pos1b = out_pos;
    ivec2 in_pos1c = out_pos;
    if(out_pos.x < pair_size.x){ 
        in_pos0b.x = out_pos.x + pair_size.x; 
        in_pos0c.x = out_pos.x + pair_size.x * 2; 
        in_pos1b.x = in_pos0b.x;
        in_pos1c.x = in_pos0c.x;
    }else if(out_pos.x < pair_size.x * 2){
        in_pos0a.x = out_pos.x - pair_size.x;  
        in_pos0c.x = out_pos.x + pair_size.x; 
        in_pos1a.x = in_pos0a.x;
        in_pos1c.x = in_pos0c.x;
    }else{
        in_pos0a.x = out_pos.x - pair_size.x * 2;  
        in_pos0b.x = out_pos.x - pair_size.x; 
        in_pos1a.x = in_pos0a.x;
        in_pos1b.x = in_pos0b.x;
    }
    if(out_pos.y < pair_size.y){
        in_pos1a.y = out_pos.y + pair_size.y;
        in_pos1b.y = in_pos1a.y;
        in_pos1c.y = in_pos1a.y;
    }else{
        in_pos0a.y = out_pos.y - pair_size.y;
        in_pos0b.y = in_pos0a.y;
        in_pos0c.y = in_pos0a.y;
    }
"##;

pub const ARROW_PALETTE: &str = r##"
    vec3 p0a = texelFetch(origin_tex, in_pos0a, 0).xyz;
    vec3 p0b = texelFetch(origin_tex, in_pos0b, 0).xyz;
    vec3 p0c = texelFetch(origin_tex, in_pos0c, 0).xyz;
    vec3 p1a = texelFetch(origin_tex, in_pos1a, 0).xyz;
    vec3 p1b = texelFetch(origin_tex, in_pos1b, 0).xyz;
    vec3 p1c = texelFetch(origin_tex, in_pos1c, 0).xyz;
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
        t0u = texelFetch(vector_tex_u, in_pos0c,  0);
        t0v = texelFetch(vector_tex_v, in_pos0c,  0);
        t1u = texelFetch(vector_tex_u, in_pos1c,  0);
        t1v = texelFetch(vector_tex_v, in_pos1c,  0);
    }else if(pick > 0){
        t0u = texelFetch(vector_tex_u, in_pos0b,  0);
        t0v = texelFetch(vector_tex_v, in_pos0b,  0);
        t1u = texelFetch(vector_tex_u, in_pos1a,  0);
        t1v = texelFetch(vector_tex_v, in_pos1a,  0);
    }else{
        t0u = texelFetch(vector_tex_u, in_pos0a,  0);
        t0v = texelFetch(vector_tex_v, in_pos0a,  0);
        t1u = texelFetch(vector_tex_u, in_pos1b,  0);
        t1v = texelFetch(vector_tex_v, in_pos1b,  0);
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
    vec3 delta = vec3(0., 0., 0.);
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
        origin   = pa;
        vector_u = vec4(du.x, du.y, du.z, uv.x);
        vector_v = vec4(dv.x, dv.y, dv.z, uv.y);
    }else{
        if(out_pos.x < pair_size.x * 2){
            delta = pb - pa;
        }else{
            delta = get_facet_convergence_point(uvs.rg, p0, d0u, d0v, uvs.ba, p1, d1u, d1v) - pa;
        }
        uv = get_moved_uv(uv, du, dv, delta);
        output_arrows(facet_index, uv);
    }
"##;

pub const INTERSECT_FACET: &str = r##"
vec3 get_facet_convergence_point(vec2 uv0, vec3 p0, vec3 d0u, vec3 d0v, vec2 uv1, vec3 p1, vec3 d1u, vec3 d1v){
    vec3 normal0 = cross(d0u, d0v);
    vec3 normal1 = cross(d1u, d1v);
    vec3 normal_cross = cross(normal0, normal1);
    vec3 cross0 = normalize(cross(normal0, normal_cross));
    vec3 cross1 = normalize(cross(normal1, normal_cross));
    return get_arrow_middle(p0, cross0, p1, cross1);
}
"##;

pub const INTERSECT_FACET_EDGE: &str = r##"
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
    return get_arrow_middle(p0, cross0, p1, cross1);
}
"##;

pub const MOVE_UV: &str = r##"
vec2 get_moved_uv(vec2 uv, vec3 du, vec3 dv, vec3 target) {
    if(isnan(target.x) || isnan(target.y) || isnan(target.z) || length(target) < 0.0001){  // 0.0001
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

pub const ARROW_CORE: &str = r##"

vec2 get_arrow_middle_2d(vec2 p0, vec2 delta0, vec2 p1, vec2 delta1, vec2 alt) {
    float dotx = dot(delta0, delta1);
    if(abs(dotx) > 0.9999) {
        return alt; 
    }
    vec2 delta = p0 - p1;
    float dot0 = dot(delta, delta0);
    float dot1 = dot(delta, delta1);
    float denom = 1. - dotx * dotx;
    float t = (dotx * dot1 - dot0) / denom;
    float s = (dot1 - dotx * dot0) / denom;
    vec2 closest0 = p0 + t * delta0;
    vec2 closest1 = p1 + s * delta1;
    return (closest0 + closest1) / 2.;
}

vec3 get_arrow_middle(vec3 p0, vec3 delta0, vec3 p1, vec3 delta1) {
    float dotx = dot(delta0, delta1);
    if(abs(dotx) > 0.9999) {
        return (p0 + p1) / 2.;
    }
    vec3 delta = p0 - p1;
    float dot0 = dot(delta, delta0);
    float dot1 = dot(delta, delta1);
    float denom = 1. - dotx * dotx;
    float t = (dotx * dot1 - dot0) / denom;
    float s = (dot1 - dotx * dot0) / denom;
    vec3 closest0 = p0 + t * delta0;
    vec3 closest1 = p1 + s * delta1;
    return (closest0 + closest1) / 2.;
}
"##;


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