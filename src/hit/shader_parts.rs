use const_format::concatcp;
use crate::{
    HIT_TOL_STR, 
    MISS_PADDING_STR, 
    AT_0_TOL_STR, AT_1_TOL_STR, 
    DUP_0_TOL_STR, DUP_1_TOL_STR, 
    DOT_1_TOL_STR,
};

pub const HEADER: &str = concatcp!(r##"#version 300 es
precision highp float;
precision highp sampler2D;
precision highp isampler2D;
uniform isampler2D index_texture;
const float HIT_TOL      = "##, HIT_TOL_STR, r##";
const float MISS_PADDING = "##, MISS_PADDING_STR, r##";
const float AT_0_TOL     = "##, AT_0_TOL_STR, r##";
const float AT_1_TOL     = "##, AT_1_TOL_STR, r##";
const float DUP_0_TOL    = "##, DUP_0_TOL_STR, r##";
const float DUP_1_TOL    = "##, DUP_1_TOL_STR, r##";
const float DOT_1_TOL    = "##, DOT_1_TOL_STR, ";"
); 

pub const GEOM_CORE: &str = r##"
uniform sampler2D shape_texture;
uniform int max_knot_count;
int shape_texture_width = 0;

float shape_texel(int index) {
    int y = index / shape_texture_width;
    int x = index % shape_texture_width;
    return texelFetch(shape_texture, ivec2(x, y), 0).r;
}

int get_knot_index(int idx, int knot_len, int order, float u){
    for(int i = 0; i < max_knot_count-1; i++) { 
        if(i < knot_len && u >= shape_texel(idx + i) && u < shape_texel(idx + i + 1)) { 
            return i; 
        }
    }
    return knot_len - order - 1;
}

float[8] get_basis(int ki, int order, int control_len, float u){
    float k0  = shape_texel(ki);
    float k1  = shape_texel(ki + 1);
    float k1u  = k1 - u;
    float uk0  = u - k0;
    float k0k1 = k0 - k1;
    float k1k0 = k1 - k0;
    float k1u_d_k1k0 = k1u / k1k0;
    float uk0_d_k1k0 = uk0 / k1k0;
    if(order > 2){ // quadratic
        float r1 = shape_texel(ki - 1);
        float k2 = shape_texel(ki + 2);
        float w0 = shape_texel(ki + control_len + 1);
        float w1 = shape_texel(ki + control_len + 2);
        float w2 = shape_texel(ki + control_len + 3);
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
        float p1 = w1 * (k1u_d_k1k0 * ur1/k1r1 + uk0_d_k1k0 * k2u/k2k0);
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

vec3 get_point(int si) {
    return vec3(
        shape_texel(si + 0),
        shape_texel(si + 1),
        shape_texel(si + 2)
    );
}

vec3 get_curve_arrow(int si, float u, out vec3 du) {
    int control_len = int(shape_texel(si + 1));
    int order = int(shape_texel(si + 2));
    float min = shape_texel(si + 3);
    float max = shape_texel(si + 4);
    int knot_len = control_len + order;
    u = min*(1.-u) + max*u;
    int ki = get_knot_index(si + 5, knot_len, order, u);
    int control_start = si + 5 + knot_len + control_len*2 + (ki-order+1)*3;
    float[8] basis = get_basis(si + 5 + ki, order, control_len, u);
    vec3 point = vec3(0., 0., 0.);
    for(int k = 0; k < order; k++) {
        vec3 control_point = get_point(control_start + k*3);
        point += control_point * basis[4-order+k];
        du    += control_point * basis[8-order+k];
    }
    du *= (max - min);
    return point; 
}
"##;

pub const CORE_PARTS: &str = r##"
ivec2 pair_size = textureSize(index_texture, 0);
ivec2 out_pos = ivec2(gl_FragCoord.x, gl_FragCoord.y);
"##; 

pub const GEOM_PARTS: &str = r##"
shape_texture_width = textureSize(shape_texture, 0).x;
"##;

pub const PALETTE_IN_POS: &str = r##"
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

pub const ARROW_HIT: &str = r##"
vec3 get_arrow_hit(vec3 p0, vec3 delta0, vec3 p1, vec3 delta1) {
    delta0 = normalize(delta0);
    delta1 = normalize(delta1);
    float dotx = dot(delta0, delta1);
    if(abs(dotx) > 0.9999) {
        return (p0 + p1) / 2.;
    }
    float denom = 1. - dotx * dotx;
    vec3 v0a = p0 * delta0 / denom        - p1 * delta0 / denom;
    vec3 v0b = p0 * delta0 * dotx / denom - p1 * delta0 * dotx / denom;
    vec3 v1a = p0 * delta1 * dotx / denom - p1 * delta1 * dotx / denom;
    vec3 v1b = p0 * delta1 / denom        - p1 * delta1 / denom;
    float dot0a = v0a.x + v0a.y + v0a.z;
    float dot0b = v0b.x + v0b.y + v0b.z;
    float dot1a = v1a.x + v1a.y + v1a.z;
    float dot1b = v1b.x + v1b.y + v1b.z;
        // float u0 = dot1a - dot0a;
        // float u1 = dot1b - dot0b;
    vec3 closest0 = p0 + delta0 * dot1a - delta0 * dot0a;               
    vec3 closest1 = p1 + delta1 * dot1b - delta1 * dot0b;
        // vec3 closest0 = p0 + delta0 * u0;               // TODO: can use this part as u or uv move directly?!
        // vec3 closest1 = p1 + delta1 * u1;
    return (closest0 + closest1) / 2.;
}
"##;





// float[6] get_curve_arrow(int si, float u) {
//     int control_len = int(shape_texel(si + 1));
//     int order = int(shape_texel(si + 2));
//     float min = shape_texel(si + 3);
//     float max = shape_texel(si + 4);
//     int knot_len = control_len + order;
//     u = min*(1.-u) + max*u;
//     float range = max - min;
//     int ki = get_knot_index(si + 5, knot_len, order, u);
//     int control_start = si + 5 + knot_len + control_len*2 + (ki-order+1)*3;
//     float[8] basis = get_basis(si + 5 + ki, order, control_len, u);
//     float[6] arrow = float[6](0., 0., 0., 0., 0., 0.);
//     for(int k = 0; k < order; k++) {
//         for(int j = 0; j < 3; j++) {
//             float control_component = shape_texel(control_start + k*3 + j);
//             arrow[j]   += control_component * basis[4-order+k];
//             arrow[j+3] += control_component * basis[8-order+k] * range;
//         }
//     }
//     return arrow; 
// }



// pub const ARROW_HIT: &str = r##"
// vec3 get_arrow_hit(vec3 p0, vec3 delta0, vec3 p1, vec3 delta1) {
//     delta0 = normalize(delta0);
//     delta1 = normalize(delta1);
//     float dotx = dot(delta0, delta1);
//     if(abs(dotx) > 0.9999) {
//         return (p0 + p1) / 2.;
//     }
//     vec3 delta = p0 - p1;
//     float dot0 = dot(delta, delta0);
//     float dot1 = dot(delta, delta1);
//     float denom = 1. - dotx * dotx;
//     float u0 = (       dotx * dot1 - dot0) / denom;
//     float u1 = (dot1 - dotx * dot0)        / denom;
//     vec3 closest0 = p0 + delta0 * u0;               // TODO: can use this part as u or uv move directly!!!
//     vec3 closest1 = p1 + delta1 * u1;
//     return (closest0 + closest1) / 2.;
// }
// "##;