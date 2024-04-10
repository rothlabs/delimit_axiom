pub const FACET_PARTS: &str = r##"
    facet_tex_width = textureSize(facet_tex, 0).x;
    ivec2 pair_size = textureSize(pair_tex, 0);
    ivec2 out_coord = ivec2(gl_FragCoord.x, gl_FragCoord.y);
"##;

pub const FACET_CORE: &str = r##"
uniform sampler2D facet_tex;
uniform isampler2D pair_tex;
uniform int max_facet_length;
uniform int max_knot_count;

int facet_tex_width = 0;
float uv_shift = 0.0001;

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

vec3 get_point_from_index(int pi){
    return vec3(get_facet_texel(pi), get_facet_texel(pi+1), get_facet_texel(pi+2));
}

int get_knot_index(int idx, int knot_count, float u){
    int knot_i = -1; 
    for(int i = 0; i < max_knot_count-1; i++) { 
        if(knot_i < 0 && i < knot_count && u >= get_facet_texel(idx + i) && u < get_facet_texel(idx + i + 1)) { 
            knot_i = i;
        }
    }
    return knot_i;
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
            //return vec4(0., p0/sum, p1/sum, p2/sum);
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

float[6] get_curve_ray(int idx, int nth, float u){
    int ci = get_curve_index(idx, nth);
    int control_count = int(get_facet_texel(ci + 1));
    int order = int(get_facet_texel(ci + 2));
    float min = get_facet_texel(ci + 3);
    float max = get_facet_texel(ci + 4);
    int knot_count = control_count + order;
    u = min*(1.-u) + max*u;
    float velocity_scale = max - min;
    int knot_i = get_knot_index(ci + 5, knot_count, u);
    if(knot_i < 0){
        if(order > 2){
            knot_i = 2;
        }else{
            knot_i = 1;
        }
    }
    // if(knot_i < 0){
    //     // return get_point_from_index(ci+5+knot_count+control_count + (control_count-1)*3);
    //     return float[6](0., 0., 0., 1., 0., 0.);
    // }else{
        int control_start = ci + 5 + knot_count + control_count + (knot_i-order+1)*3;
        float[8] basis = get_basis(ci + 5 + knot_i, order, control_count, u);
        float[6] ray = float[6](0., 0., 0., 0., 0., 0.);
        for(int k = 0; k < order; k++) {
            for(int j = 0; j < 3; j++) {
                float control_component = get_facet_texel(control_start + k*3 + j);
                ray[j]   += control_component * basis[4-order+k];
                ray[j+3] += control_component * basis[8-order+k] * velocity_scale;
            }
        }
        return ray; 
    //}
}

float[9] get_facet_rays(int fi, vec2 uv){
    int control_count = int(get_facet_texel(fi + 1));
    int order = int(get_facet_texel(fi + 2));
    int knot_count = control_count + order;
    int knot_i = get_knot_index(fi + 3, knot_count, uv.y);
    if(knot_i < 0){
        if(order > 2){
            knot_i = 2;
        }else{
            knot_i = 1;
        }
    }
    // if(knot_i < 0){
    //     // return get_point_on_curve(fi, control_count-1, uv.x);
    //     float[9](0., 0., 0., 1., 0., 0., 1., 0., 0.);
    // }else{
        int control_start = knot_i - order + 1;
        float[8] basis = get_basis(fi + 3 + knot_i, order, control_count, uv.y);
        float[9] rays = float[9](0., 0., 0., 0., 0., 0., 0., 0., 0.);
        for(int k = 0; k < order; k++) {
            float[6] ray = get_curve_ray(fi, control_start + k, uv.x); 
            for(int j = 0; j < 3; j++) {
                rays[j]   += ray[j]   * basis[4-order+k];
                rays[j+3] += ray[j+3] * basis[4-order+k];
                rays[j+6] += ray[j]   * basis[8-order+k];
            }
        }
        return rays; 
    //}
}
"##;


pub const HONE_CORE: &str = r##"
uniform sampler2D point_tex;
uniform sampler2D deriv_tex_u;
uniform sampler2D deriv_tex_v;
layout(location=0) out vec3 point;
layout(location=1) out vec4 deriv_u;
layout(location=2) out vec4 deriv_v;
"##;

pub const GET_RAY_DUAL: &str = r##"
    ivec2 in_coord0 = out_coord;
    ivec2 in_coord1 = out_coord;
    if(!(out_coord.x < pair_size.x)){ 
        in_coord0.x = out_coord.x - pair_size.x; 
        in_coord1.x = in_coord0.x;
    }
    if(out_coord.y < pair_size.y){
        in_coord1.y = out_coord.y + pair_size.y;
    }else{
        in_coord0.y = out_coord.y - pair_size.y;
    }
    ivec2 facet_i = texelFetch(pair_tex, in_coord0, 0).rg;
    vec4 t0u = texelFetch(deriv_tex_u, in_coord0, 0);
    vec4 t0v = texelFetch(deriv_tex_v, in_coord0, 0);
    vec4 t1u = texelFetch(deriv_tex_u, in_coord1, 0);
    vec4 t1v = texelFetch(deriv_tex_v, in_coord1, 0);
    vec4 uvs = vec4(t0u.a, t0v.a,  t1u.a, t1v.a);
    vec3 p0  = texelFetch(point_tex, in_coord0, 0).xyz;
    vec3 p1  = texelFetch(point_tex, in_coord1, 0).xyz;
    vec3 d0u = t0u.xyz;
    vec3 d0v = t0v.xyz;
    vec3 d1u = t1u.xyz;
    vec3 d1v = t1v.xyz;
"##;

pub const GET_RAY_QUAD: &str = r##"
    ivec2 in_coord0 = out_coord;
    ivec2 in_coord1 = out_coord;
    ivec2 in_coord2 = out_coord;
    ivec2 in_coord3 = out_coord;
    if(out_coord.x < pair_size.x){ 
        in_coord1.x = out_coord.x + pair_size.x; 
        in_coord3.x = in_coord1.x;
    }else{
        in_coord0.x = out_coord.x - pair_size.x; 
        in_coord2.x = in_coord0.x;
    }
    if(out_coord.y < pair_size.y){
        in_coord2.y = out_coord.y + pair_size.y;
        in_coord3.y = in_coord2.y;
    }else{
        in_coord0.y = out_coord.y - pair_size.y;
        in_coord1.y = in_coord0.y;
    }
    ivec2 facet_i = texelFetch(pair_tex, in_coord0, 0).rg;

    vec3 p0  = texelFetch(point_tex, in_coord0,  0).xyz;
    vec3 p0b = texelFetch(point_tex, in_coord1,  0).xyz;
    vec4 t0u = vec4(0., 0., 0., 0.);
    vec4 t0v = vec4(0., 0., 0., 0.);
    vec3 p1  = texelFetch(point_tex, in_coord2,  0).xyz;
    vec3 p1b = texelFetch(point_tex, in_coord3,  0).xyz;
    vec4 t1u = vec4(0., 0., 0., 0.);
    vec4 t1v = vec4(0., 0., 0., 0.);
    
    int pick = 0;
    vec3 p0_ = p0;
    if(length(p0b - p1) < length(p0 - p1)){
        pick = 1;
        p0 = p0b;
    }
    if(length(p0 - p1b) < length(p0 - p1)){
        pick = 2;
        p0 = p0_;
        p1 = p1b;
    }
    if(pick > 1){
        t0u = texelFetch(deriv_tex_u, in_coord0,  0);
        t0v = texelFetch(deriv_tex_v, in_coord0,  0);
        t1u = texelFetch(deriv_tex_u, in_coord3,  0);
        t1v = texelFetch(deriv_tex_v, in_coord3,  0);
    }else if(pick > 0){
        t0u = texelFetch(deriv_tex_u, in_coord1,  0);
        t0v = texelFetch(deriv_tex_v, in_coord1,  0);
        t1u = texelFetch(deriv_tex_u, in_coord2,  0);
        t1v = texelFetch(deriv_tex_v, in_coord2,  0);
    }else{
        t0u = texelFetch(deriv_tex_u, in_coord0,  0);
        t0v = texelFetch(deriv_tex_v, in_coord0,  0);
        t1u = texelFetch(deriv_tex_u, in_coord2,  0);
        t1v = texelFetch(deriv_tex_v, in_coord2,  0);
    }
    vec3 d0u = t0u.xyz;
    vec3 d0v = t0v.xyz;
    vec3 d1u = t1u.xyz;
    vec3 d1v = t1v.xyz;
    vec4 uvs = vec4(t0u.a, t0v.a, t1u.a, t1v.a);
"##;

pub const HONE: &str = r##"
int fi = 0;
vec2 uv_in  = vec2(0., 0.);
vec3 pnt  = vec3(0., 0., 0.);
vec3 pdu    = vec3(0., 0., 0.);
vec3 pdv    = vec3(0., 0., 0.);
vec3 delta  = vec3(0., 0., 0.);
if(out_coord.y < pair_size.y){
    fi = facet_i.r; uv_in = uvs.rg; pnt = p0; pdu = d0u; pdv=d0v;
    delta = p1 - pnt;
}else{
    fi = facet_i.g; uv_in = uvs.ba; pnt = p1; pdu = d1u; pdv=d1v;
    delta = p0 - pnt;
}
if(out_coord.x < pair_size.x){
    delta = get_point_between_facet_tangents(p0, d0u, d0v, p1, d1u, d1v) - pnt;
}
vec2 uv_out = get_uv_from_3d_delta(uv_in, pdu, pdv, delta);
float[9] rays = get_facet_rays(fi, uv_out);
point   = vec3(rays[0], rays[1], rays[2]);
deriv_u = vec4(rays[3], rays[4], rays[5], uv_out.x);
deriv_v = vec4(rays[6], rays[7], rays[8], uv_out.y);
"##;

// pub const RAY_OUT: &str = r##"
//     point   = vec3(rays[0], rays[1], rays[2]);
//     deriv_u = vec3(rays[3], rays[4], rays[5]);
//     deriv_v = vec3(rays[6], rays[7], rays[8]);
// "##;

pub const RAY_CORE: &str = r##"

vec2 get_line_intersection(vec2 alt, vec2 p1, vec2 p2, vec2 p3, vec2 p4) {
    float u = - ((p1.x - p2.x)*(p1.y - p3.y) - (p1.y - p2.y)*(p1.x - p3.x))
              / ((p1.x - p2.x)*(p3.y - p4.y) - (p1.y - p2.y)*(p3.x - p4.x));
    float x = p3.x + u * (p4.x - p3.x);
    float y = p3.y + u * (p4.y - p3.y);
    if(isnan(x) || isnan(y)){
        return alt;
    }
    return vec2(x, y);
}

vec3 get_point_between_lines(vec3 p0, vec3 d1, vec3 p1, vec3 d2) {
    if(dot(normalize(d1), normalize(d2)) > 0.99) {
        return (p0 + p1) / 2.0;
    }
    float a = dot(d1, d1);
    float b = dot(d1, d2);
    float c = dot(d2, d2);
    vec3 v = p0 - p1;
    float d = dot(d1, v);
    float e = dot(d2, v);
    float denom = a * c - b * b;
    float t = (b * e - c * d) / denom;
    float s = (a * e - b * d) / denom;
    vec3 closest0 = p0 + t * d1;
    vec3 closest1 = p1 + s * d2;
    return (closest0 + closest1) / 2.;
}

vec3 get_point_between_facet_tangents(vec3 p0, vec3 d0u, vec3 d0v, vec3 p1, vec3 d1u, vec3 d1v){
    vec3 normal0 = normalize(cross(d0u, d0v));
    vec3 normal1 = normalize(cross(d1u, d1v));
    vec3 normal_cross = normalize(cross(normal0, normal1));
    vec3 cross0 = normalize(cross(normal0, normal_cross));
    vec3 cross1 = normalize(cross(normal1, normal_cross));
    return get_point_between_lines(p0, cross0, p1, cross1);
}

vec2 get_uv_from_3d_delta(vec2 uv_in, vec3 pdu, vec3 pdv, vec3 target) {
    if(isnan(target.x) || isnan(target.y) || isnan(target.z) || length(target) < 0.0001){
        return uv_in;
    }
    float length_ratio_u = length(target) / length(pdu);
    float length_ratio_v = length(target) / length(pdv);
    vec2 uv_delta = vec2(
        dot(normalize(pdu), normalize(target)) * length_ratio_u, 
        dot(normalize(pdv), normalize(target)) * length_ratio_v
    );
    vec2 uv = uv_in + uv_delta;
    if(uv.x > 1. && abs(dot(normalize(uv_delta), vec2(0., 1.))) < 0.95){
        uv = get_line_intersection(uv, uv_in, uv_in + uv_delta*100., vec2(1., 0.), vec2(1., 1.));
    }else if(uv.x < 0. && abs(dot(normalize(uv_delta), vec2(0., 1.))) < 0.95){
        uv = get_line_intersection(uv, uv_in, uv_in + uv_delta*100., vec2(0., 0.), vec2(0., 1.));
    }
    if(uv.y > 1. && abs(dot(normalize(uv_delta), vec2(1., 0.))) < 0.95){
        uv = get_line_intersection(uv, uv_in, uv_in + uv_delta*100., vec2(0., 1.), vec2(1., 1.));
    }else if(uv.y < 0. && abs(dot(normalize(uv_delta), vec2(1., 0.))) < 0.95){
        uv = get_line_intersection(uv, uv_in, uv_in + uv_delta*100., vec2(0., 0.), vec2(1., 0.));
    }
    uv.x = clamp(uv.x, 0., 1.);
    uv.y = clamp(uv.y, 0., 1.);
    return uv;
}
"##;

// pub const HONE_PARTS: &str = r##"
//     vec3 center = get_point_between_facet_tangents(uv0, p0a, p0b, p0c, uv1, p1a, p1b, p1c);
//     vec2 uv0_a = get_uv_from_3d_delta(uv0, p0a, p0b, p0c, center - p0a);
//     vec3 p0_a  = get_point_on_facet(facet_i.r, uv0_a);
//     vec2 uv1_a = get_uv_from_3d_delta(uv1, p1a, p1b, p1c, center - p1a);
//     vec3 p1_a  = get_point_on_facet(facet_i.g, uv1_a);
    
//     vec2 uv0_c = get_uv_from_3d_delta(uv0, p0a, p0b, p0c, p1a - p0a);
//     vec3 p0_c  = get_point_on_facet(facet_i.r, uv0_c);
//     vec2 uv1_c = get_uv_from_3d_delta(uv1, p1a, p1b, p1c, p0a - p1a);
//     vec3 p1_c  = get_point_on_facet(facet_i.g, uv1_c);
//     vec3 lengths = vec3(
//         length(p0_a - p1_a), 
//         //length(p0_b - p1_b),
//         length(p1a  - p0_c),
//         length(p0a  - p1_c)
//     );
//     float min_dist = 10000.;
//     int i = 0;
//     for(int k = 0; k < 3; k++){
//         if(min_dist > lengths[k]){
//             min_dist = lengths[k];
//             i = k;
//         }
//     }
// "##;



// pub const UV_POINT_DERIV_PAIR: &str = r##"
//     ivec2 facet_i = texelFetch(pair_tex, out_coord, 0).rg;
//     vec4 uvs = texelFetch(uv_tex, out_coord, 0);
//     vec3 p0  = texelFetch(point_tex,   out_coord, 0).rgb;
//     vec3 d0u = texelFetch(deriv_tex_u, out_coord, 0).rgb;
//     vec3 d0v = texelFetch(deriv_tex_v, out_coord, 0).rgb;
//     ivec2 out_coord1 = ivec2(out_coord.x, out_coord.y + pair_size.y);
//     vec3 p1  = texelFetch(point_tex,   out_coord1, 0).rgb;
//     vec3 d1u = texelFetch(deriv_tex_u, out_coord1, 0).rgb;
//     vec3 d1v = texelFetch(deriv_tex_v, out_coord1, 0).rgb;
// "##;



// pub const UV_POINT_PARTS: &str = r##"
//     ivec2 facet_i = texelFetch(pair_tex, out_coord, 0).rg;
//     vec2 uv0 = texelFetch(uv_tex, out_coord, 0).rg;
//     vec2 uv1 = texelFetch(uv_tex, out_coord, 0).ba;
//     vec3 p0  = texelFetch(point_tex,       out_coord,                                  0).rgb;
//     vec3 d0u = texelFetch(deriv_tex_u, ivec2(out_coord.x + pair_size.x,   out_coord.y), 0).rgb;
//     vec3 d0v = texelFetch(deriv_tex_v, ivec2(out_coord.x + pair_size.x*2, out_coord.y), 0).rgb;
//     vec3 p1a = texelFetch(point_tex, ivec2(out_coord.x,                 out_coord.y + pair_size.y), 0).rgb;
//     vec3 p1b = texelFetch(point_tex, ivec2(out_coord.x + pair_size.x,   out_coord.y + pair_size.y), 0).rgb;
//     vec3 p1c = texelFetch(point_tex, ivec2(out_coord.x + pair_size.x*2, out_coord.y + pair_size.y), 0).rgb;
// "##;


// vec3 get_facet_normal(vec2 uv, vec3 p0, vec3 p1, vec3 p2){
//     float s = 1.;
//     if(uv.x > 0.5) s = -s;
//     if(uv.y > 0.5) s = -s;
//     return normalize(cross(p0 - p1, p0 - p2)) * s;
// }


// vec4 get_linear_position_basis(int ki, float u) {
//     float k0  = get_facet_texel(ki);
//     float k1  = get_facet_texel(ki + 1);
//     return vec4(0., 0., k1u/k1k0, uk0/k1k0);
// }

// vec4 get_quadratic_position_basis(int ki, float u) {
//     float k0  = get_facet_texel(ki);
//     float k1  = get_facet_texel(ki + 1);
//     return vec4(0., 0., k1u/k1k0, uk0/k1k0);
// }

// vec4 get_velocity_basis(int ki, int order, int control_len, float u){

// }



// float get_rational_basis_sum(int idx, vec4 basis, int order){
//     float sum = 0.;
//     for(int k = 0; k < order; k++) {
//         sum += basis[4-order+k] * get_facet_texel(idx+k);
//     }
//     return sum;
// }


// vec4 get_basis(int idx, int order, float u){
//     vec4 basis = vec4(0., 0., 0., 1.);
//     for(int degree = 1; degree < order; degree++) {
//         for(int k = 0; k < degree+1; k++) { 
//             int i = 3 - degree + k;
//             int i0 = idx + k - degree;
//             int i1 = i0 + 1;  
//             float interpolation = 0.;
//             if(basis[i] != 0.) {
//                 float knot0  = get_facet_texel(i0);
//                 float knot0d = get_facet_texel(i0 + degree);
//                 interpolation += basis[i] * (u - knot0) / (knot0d - knot0); 
//             }
//             if(i < 3 && basis[i+1] != 0.) {
//                 float knot1  = get_facet_texel(i1);
//                 float knot1d = get_facet_texel(i1 + degree);
//                 interpolation += basis[i+1] * (knot1d - u) / (knot1d - knot1); 
//             }
//             basis[i] = interpolation;
//         }
//     }
//     return basis;
// }


// vec3 get_point_on_facet_old(int fi, vec2 uv){
//     int control_count = int(get_facet_texel(fi + 1));
//     int order = int(get_facet_texel(fi + 2));
//     int knot_count = control_count + order;
//     int knot_i = get_knot_index(fi + 3, knot_count, uv.y);
//     if(knot_i < 0){
//         return get_point_on_curve(fi, control_count-1, uv.x);
//     }else{
//         int weight_start = fi + 3 + knot_i + control_count + 1;
//         int control_start = knot_i - order + 1;
//         vec4 basis = get_basis(fi + 3 + knot_i, order, uv.y);
//         float sum = get_rational_basis_sum(weight_start, basis, order);
//         vec3 point = vec3(0., 0., 0.);
//         for(int k = 0; k < order; k++) {
//             float basis = basis[4-order+k] * get_facet_texel(weight_start+k) / sum;
//             point += get_point_on_curve(fi, control_start+k, uv.x) * basis;
//         }
//         return point; 
//     }
// }


// center = (p0a + p1a) / 2.;
    // vec2 uv0_b = get_uv_from_3d_delta(uv0, p0a, p0b, p0c, center - p0a);
    // vec3 p0_b  = get_point_on_facet(facet_i.r, uv0_b);
    // vec2 uv1_b = get_uv_from_3d_delta(uv1, p1a, p1b, p1c, center - p1a);
    // vec3 p1_b  = get_point_on_facet(facet_i.g, uv1_b);


    // vec2 get_uv_from_3d_delta(vec2 uv, vec3 p0, vec3 p1, vec3 p2, vec3 target) {
    //     if(isnan(target.x) || isnan(target.y) || isnan(target.z) || length(target) < 0.0001){
    //         return uv;
    //     }
    //     float su = uv_shift;
    //     float sv = uv_shift;
    //     if(uv.x > 0.5) su = -su;
    //     if(uv.y > 0.5) sv = -sv;
    //     float length_ratio_u = length(target) / length(p0-p1) * su;
    //     float length_ratio_v = length(target) / length(p0-p2) * sv;
    //     vec2 uv_delta = vec2(
    //         dot(normalize(p1-p0), normalize(target)) * length_ratio_u, 
    //         dot(normalize(p2-p0), normalize(target)) * length_ratio_v
    //     );
    //     vec2 uv1 = uv + uv_delta;
    //     if(uv1.x > 1. && abs(dot(normalize(uv_delta), vec2(0., 1.))) < 0.95){
    //         uv1 = get_line_intersection(uv1, uv, uv + uv_delta*100., vec2(1., 0.), vec2(1., 1.));
    //     }else if(uv1.x < 0. && abs(dot(normalize(uv_delta), vec2(0., 1.))) < 0.95){
    //         uv1 = get_line_intersection(uv1, uv, uv + uv_delta*100., vec2(0., 0.), vec2(0., 1.));
    //     }
    //     if(uv1.y > 1. && abs(dot(normalize(uv_delta), vec2(1., 0.))) < 0.95){
    //         uv1 = get_line_intersection(uv1, uv, uv + uv_delta*100., vec2(0., 1.), vec2(1., 1.));
    //     }else if(uv1.y < 0. && abs(dot(normalize(uv_delta), vec2(1., 0.))) < 0.95){
    //         uv1 = get_line_intersection(uv1, uv, uv + uv_delta*100., vec2(0., 0.), vec2(1., 0.));
    //     }
    //     uv1.x = clamp(uv1.x, 0., 1.);
    //     uv1.y = clamp(uv1.y, 0., 1.);
    //     return uv1;
    // }