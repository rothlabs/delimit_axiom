pub const HEADER: &str = r##"#version 300 es
precision highp float;
precision highp sampler2D;
precision highp isampler2D;
uniform isampler2D pair_tex;
"##; 

pub const GEOM_CORE: &str = r##"
uniform sampler2D geom_tex;
uniform int max_knot_count;
int geom_tex_width = 0;

float get_geom_texel(int index) {
    int y = index / geom_tex_width;
    int x = index % geom_tex_width;
    return texelFetch(geom_tex, ivec2(x, y), 0).r;
}

int get_knot_index(int idx, int knot_count, int order, float u){
    for(int i = 0; i < max_knot_count-1; i++) { 
        if(i < knot_count && u >= get_geom_texel(idx + i) && u < get_geom_texel(idx + i + 1)) { 
            return i; // knot_i = i;
        }
    }
    return knot_count - order - 1;
}

float[8] get_basis(int ki, int order, int control_len, float u){
    float k0  = get_geom_texel(ki);
    float k1  = get_geom_texel(ki + 1);
    float k1u  = k1 - u;
    float uk0  = u - k0;
    float k0k1 = k0 - k1;
    float k1k0 = k1 - k0;
    float k1u_d_k1k0 = k1u / k1k0;
    float uk0_d_k1k0 = uk0 / k1k0;
    if(order > 2){ // quadratic
        float r1 = get_geom_texel(ki - 1);
        float k2 = get_geom_texel(ki + 2);
        float w0 = get_geom_texel(ki + control_len + 1);
        float w1 = get_geom_texel(ki + control_len + 2);
        float w2 = get_geom_texel(ki + control_len + 3);
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

float[6] get_curve_arrow(int ci, float u){
    int control_count = int(get_geom_texel(ci + 1));
    int order = int(get_geom_texel(ci + 2));
    float min = get_geom_texel(ci + 3);
    float max = get_geom_texel(ci + 4);
    int knot_count = control_count + order;
    u = min*(1.-u) + max*u;
    float velocity_scale = max - min;
    int knot_i = get_knot_index(ci + 5, knot_count, order, u);
    int control_start = ci + 5 + knot_count + control_count + (knot_i-order+1)*3;
    float[8] basis = get_basis(ci + 5 + knot_i, order, control_count, u);
    float[6] arrow = float[6](0., 0., 0., 0., 0., 0.);
    for(int k = 0; k < order; k++) {
        for(int j = 0; j < 3; j++) {
            float control_component = get_geom_texel(control_start + k*3 + j);
            arrow[j]   += control_component * basis[4-order+k];
            arrow[j+3] += control_component * basis[8-order+k] * velocity_scale;
        }
    }
    return arrow; 
}
"##;

pub const CORE_PARTS: &str = r##"
ivec2 pair_size = textureSize(pair_tex, 0);
ivec2 out_pos = ivec2(gl_FragCoord.x, gl_FragCoord.y);
"##; 

pub const GEOM_PARTS: &str = r##"
geom_tex_width = textureSize(geom_tex, 0).x;
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