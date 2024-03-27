pub const FACET_PARTS: &str = r##"
    ivec2 pair_size = textureSize(pair_tex, 0);
    ivec2 pair_coord = ivec2(gl_FragCoord.x, gl_FragCoord.y);
"##;

pub const FACET_CORE: &str = r##"
uniform sampler2D facet_tex;
uniform int max_facet_length;
uniform int max_knot_count;

float uv_shift = 0.0001;

float get_facet_texel(int index) {
    int width = textureSize(facet_tex, 0).x;  // size of mip 0
    int y = index / width;
    int x = index % width;
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

vec4 get_basis(int idx, int order, float u){
    vec4 basis = vec4(0., 0., 0., 1.);
    for(int degree = 1; degree < order; degree++) {
        for(int k = 0; k < degree+1; k++) { 
            int i = 3 - degree + k;
            int i0 = idx + k - degree;
            int i1 = i0 + 1;  
            float interpolation = 0.;
            if(basis[i] != 0.) {
                float knot0  = get_facet_texel(i0);
                float knot0d = get_facet_texel(i0 + degree);
                interpolation += basis[i] * (u - knot0) / (knot0d - knot0); 
            }
            if(i < 3 && basis[i+1] != 0.) {
                float knot1  = get_facet_texel(i1);
                float knot1d = get_facet_texel(i1 + degree);
                interpolation += basis[i+1] * (knot1d - u) / (knot1d - knot1); 
            }
            basis[i] = interpolation;
        }
    }
    return basis;
}

float get_rational_basis_sum(int idx, vec4 basis, int order){
    float sum = 0.;
    for(int k = 0; k < order; k++) {
        sum += basis[4-order+k] * get_facet_texel(idx+k);
    }
    return sum;
}

vec3 get_point_on_curve(int idx, int nth, float u){
    int ci = get_curve_index(idx, nth);
    int control_count = int(get_facet_texel(ci + 1));
    int order = int(get_facet_texel(ci + 2));
    float min = get_facet_texel(ci + 3);
    float max = get_facet_texel(ci + 4);
    int knot_count = control_count + order;
    u = min*(1.-u) + max*u;
    int knot_i = get_knot_index(ci + 5, knot_count, u);
    if(knot_i < 0){
        return get_point_from_index(ci+5+knot_count+control_count + (control_count-1)*3);
    }else{
        int weight_start = ci + 5 + knot_i + control_count + 1;
        int control_start = ci + 5 + knot_count + control_count + (knot_i-order+1)*3;
        vec4 basis = get_basis(ci + 5 + knot_i, order, u);
        float sum = get_rational_basis_sum(weight_start, basis, order);
        vec3 point = vec3(0., 0., 0.);
        for(int k = 0; k < order; k++) {
            float basis = basis[4-order+k] * get_facet_texel(weight_start+k) / sum;
            for(int j = 0; j < 3; j++) {
                point[j] += get_facet_texel(control_start + k*3 + j) * basis;
            }
        }
        return point; 
    }
}

vec3 get_point_on_facet(int fi, vec2 uv){
    int control_count = int(get_facet_texel(fi + 1));
    int order = int(get_facet_texel(fi + 2));
    int knot_count = control_count + order;
    int knot_i = get_knot_index(fi + 3, knot_count, uv.y);
    if(knot_i < 0){
        return get_point_on_curve(fi, control_count-1, uv.x);
    }else{
        int weight_start = fi + 3 + knot_i + control_count + 1;
        int control_start = knot_i - order + 1;
        vec4 basis = get_basis(fi + 3 + knot_i, order, uv.y);
        float sum = get_rational_basis_sum(weight_start, basis, order);
        vec3 point = vec3(0., 0., 0.);
        for(int k = 0; k < order; k++) {
            float basis = basis[4-order+k] * get_facet_texel(weight_start+k) / sum;
            point += get_point_on_curve(fi, control_start+k, uv.x) * basis;
        }
        return point; 
    }
}
"##;


pub const UV_POINT_PARTS: &str = r##"
    ivec2 facet_i = texelFetch(pair_tex, pair_coord, 0).rg;
    vec2 uv0 = texelFetch(uv_tex, pair_coord, 0).rg;
    vec2 uv1 = texelFetch(uv_tex, pair_coord, 0).ba;
    vec3 p0a = texelFetch(point_tex,       pair_coord,                                  0).rgb;
    vec3 p0b = texelFetch(point_tex, ivec2(pair_coord.x + pair_size.x,   pair_coord.y), 0).rgb;
    vec3 p0c = texelFetch(point_tex, ivec2(pair_coord.x + pair_size.x*2, pair_coord.y), 0).rgb;
    vec3 p1a = texelFetch(point_tex, ivec2(pair_coord.x,                 pair_coord.y + pair_size.y), 0).rgb;
    vec3 p1b = texelFetch(point_tex, ivec2(pair_coord.x + pair_size.x,   pair_coord.y + pair_size.y), 0).rgb;
    vec3 p1c = texelFetch(point_tex, ivec2(pair_coord.x + pair_size.x*2, pair_coord.y + pair_size.y), 0).rgb;
"##;

pub const UV_POINT_CORE: &str = r##"
vec3 get_facet_normal(vec2 uv, vec3 p0, vec3 p1, vec3 p2){
    float s = 1.;
    if(uv.x > 0.5) s = -s;
    if(uv.y > 0.5) s = -s;
    return normalize(cross(p0 - p1, p0 - p2)) * s;
}

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
    vec3 v = p0 - p1;
    float a = dot(d1, d1);
    float b = dot(d1, d2);
    float c = dot(d2, d2);
    float d = dot(d1, v);
    float e = dot(d2, v);
    float denom = a * c - b * b;
    float t = (b * e - c * d) / denom;
    float s = (a * e - b * d) / denom;
    vec3 closest0 = p0 + t * d1;
    vec3 closest1 = p1 + s * d2;
    return (closest0 + closest1) / 2.;
}

vec3 get_point_between_facet_tangents(vec2 uv0, vec3 p0a, vec3 p0b, vec3 p0c, vec2 uv1, vec3 p1a, vec3 p1b, vec3 p1c){
    vec3 normal0 = get_facet_normal(uv0, p0a, p0b, p0c);
    vec3 normal1 = get_facet_normal(uv1, p1a, p1b, p1c);
    vec3 normal_cross = normalize(cross(normal0, normal1));
    vec3 cross0 = normalize(cross(normal0, normal_cross));
    vec3 cross1 = normalize(cross(normal1, normal_cross));
    return get_point_between_lines(p0a, cross0, p1a, cross1);
}

vec2 get_uv_from_3d_move_target(vec2 uv, vec3 p0, vec3 p1, vec3 p2, vec3 target) {
    if(isnan(target.x) || isnan(target.y) || isnan(target.z) || length(target) < 0.0001){
        return uv;
    }
    float su = uv_shift;
    float sv = uv_shift;
    if(uv.x > 0.5) su = -su;
    if(uv.y > 0.5) sv = -sv;
    float length_ratio_u = length(target) / length(p0-p1) * su;
    float length_ratio_v = length(target) / length(p0-p2) * sv;
    vec2 uv_delta = vec2(
        dot(normalize(p1-p0), normalize(target)) * length_ratio_u, 
        dot(normalize(p2-p0), normalize(target)) * length_ratio_v
    );
    vec2 uv1 = uv + uv_delta;
    if(uv1.x > 1. && abs(dot(normalize(uv_delta), vec2(0., 1.))) < 0.95){
        uv1 = get_line_intersection(uv1, uv, uv + uv_delta*100., vec2(1., 0.), vec2(1., 1.));
    }else if(uv1.x < 0. && abs(dot(normalize(uv_delta), vec2(0., 1.))) < 0.95){
        uv1 = get_line_intersection(uv1, uv, uv + uv_delta*100., vec2(0., 0.), vec2(0., 1.));
    }
    if(uv1.y > 1. && abs(dot(normalize(uv_delta), vec2(1., 0.))) < 0.95){
        uv1 = get_line_intersection(uv1, uv, uv + uv_delta*100., vec2(0., 1.), vec2(1., 1.));
    }else if(uv1.y < 0. && abs(dot(normalize(uv_delta), vec2(1., 0.))) < 0.95){
        uv1 = get_line_intersection(uv1, uv, uv + uv_delta*100., vec2(0., 0.), vec2(1., 0.));
    }
    uv1.x = clamp(uv1.x, 0., 1.);
    uv1.y = clamp(uv1.y, 0., 1.);
    return uv1;
}
"##;

pub const HONE_PARTS: &str = r##"
    vec3 center = get_point_between_facet_tangents(uv0, p0a, p0b, p0c, uv1, p1a, p1b, p1c);
    vec2 uv0_a = get_uv_from_3d_move_target(uv0, p0a, p0b, p0c, center - p0a);
    vec3 p0_a  = get_point_on_facet(facet_i.r, uv0_a);
    vec2 uv1_a = get_uv_from_3d_move_target(uv1, p1a, p1b, p1c, center - p1a);
    vec3 p1_a  = get_point_on_facet(facet_i.g, uv1_a);
    
    center = (p0a + p1a) / 2.;
    vec2 uv0_b = get_uv_from_3d_move_target(uv0, p0a, p0b, p0c, center - p0a);
    vec3 p0_b  = get_point_on_facet(facet_i.r, uv0_b);
    vec2 uv1_b = get_uv_from_3d_move_target(uv1, p1a, p1b, p1c, center - p1a);
    vec3 p1_b  = get_point_on_facet(facet_i.g, uv1_b);
    
    vec2 uv0_c = get_uv_from_3d_move_target(uv0, p0a, p0b, p0c, p1a - p0a);
    vec3 p0_c  = get_point_on_facet(facet_i.r, uv0_c);
    vec2 uv1_c = get_uv_from_3d_move_target(uv1, p1a, p1b, p1c, p0a - p1a);
    vec3 p1_c  = get_point_on_facet(facet_i.g, uv1_c);
    vec4 lengths = vec4(
        length(p0_a - p1_a), 
        length(p0_b - p1_b),
        length(p1a  - p0_c),
        length(p0a  - p1_c)
    );
    float min_dist = 10000.;
    int i = 3;
    for(int k = 0; k < 4; k++){
        if(min_dist > lengths[k]){
            min_dist = lengths[k];
            i = k;
        }
    }
"##;