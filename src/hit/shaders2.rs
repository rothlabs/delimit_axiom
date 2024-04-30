use const_format::concatcp;

use super::shader_parts::{
    HEADER, CORE_PARTS, GEOM_CORE, GEOM_PARTS,
    PALETTE_IN_POS,
    ARROW_HIT,
};
use super::shader_parts2::{
    ARROW_PALETTE, ARROW_IN, ARROW_OUT, MOVE_U,
};

pub const INIT_PALETTE_SOURCE: &str = concatcp!(
HEADER, GEOM_CORE, ARROW_OUT, r##"
uniform sampler2D io_tex;
void main() {"##,
    CORE_PARTS, GEOM_PARTS, 
    r##"
    int curve_index = 0;
    float u = 0.;
    ivec2 in_pos = out_pos;
    if(!(in_pos.x < pair_size.x)){
        in_pos.x = in_pos.x - pair_size.x; 
    }
    if(!(in_pos.x < pair_size.x)){
        in_pos.x = in_pos.x - pair_size.x; 
    }
    if(in_pos.y < pair_size.y){
        curve_index = texelFetch(pair_tex, in_pos, 0).r;
        u = texelFetch(io_tex, in_pos, 0).r;
    }else{
        in_pos.y = in_pos.y - pair_size.y;
        curve_index = texelFetch(pair_tex, in_pos, 0).g;
        u = texelFetch(io_tex, in_pos, 0).g;
    }
    output_arrow(curve_index, u);
}
"##);

pub const HONE_SOURCE: &str = concatcp!(
HEADER, GEOM_CORE, ARROW_HIT, MOVE_U, ARROW_IN, ARROW_OUT,
"void main() {",
    CORE_PARTS, GEOM_PARTS, PALETTE_IN_POS, ARROW_PALETTE, r##"
    int curve_index = 0;
    float u = 0.;
    vec3 du = vec3(0., 0., 0.);
    vec3 pa = vec3(0., 0., 0.);
    vec3 pb = vec3(0., 0., 0.);
    vec3 pt = vec3(0., 0., 0.);
    if(out_pos.y < pair_size.y){
        curve_index = texelFetch(pair_tex, in_pos0a, 0).r;
        u = u0; du = d0u; 
        pa = p0; pb = p1;
    }else{
        curve_index = texelFetch(pair_tex, in_pos0a, 0).g;
        u = u1; du = d1u; 
        pa = p1; pb = p0;
    }
    if(out_pos.x < pair_size.x){
        point = pa.xy;
        delta = vec4(du.x, du.y, 0., u);
    }else{
        if(out_pos.x < pair_size.x * 2){
            pt = pb;
        }else{
            pt = get_arrow_hit(p0, d0u, p1, d1u);
        }
        u = get_moved_u(u, du, pa, pt);
        output_arrow(curve_index, u);
    }
}"##);

pub const HIT_MISS_SOURCE: &str = concatcp!(
HEADER, GEOM_CORE, ARROW_IN, r##"
vec3 vec_z = vec3(0., 0., 1.);
layout(location=0) out vec4 hit_miss;
layout(location=1) out vec3 point;
void main() {"##,
    CORE_PARTS, GEOM_PARTS, PALETTE_IN_POS, ARROW_PALETTE, r##"
    d0u = normalize(d0u);
    d1u = normalize(d1u);
    if(length(p0 - p1) < HIT_TOL){
        hit_miss = vec4(-10., -10., -10., -10.); 
        if((u0 > DUP_1_TOL && u1 < DUP_0_TOL) || (u0 < DUP_0_TOL && u1 > DUP_1_TOL)){
            return;
        }
        if(abs(dot(d0u, d1u)) > DOT_1_TOL){     
            return;
        }
        vec3 cross0 = normalize(cross(d0u, vec_z));
        vec3 cross1 = normalize(cross(d1u, vec_z));
        hit_miss = vec4(
            u0,
            u1,
            dot(cross0, d1u), 
            dot(cross1, d0u)
        );
        point = (p0 + p1) / 2.;
    }else{
        if(u0 < AT_0_TOL){
            p0 = p0 + d0u * MISS_PADDING;
        }else if(u0 > AT_1_TOL){
            p0 = p0 - d0u * MISS_PADDING;
        }
        if(u1 < AT_0_TOL){
            p1 = p1 + d1u * MISS_PADDING;
        }else if(u1 > AT_1_TOL){
            p1 = p1 - d1u * MISS_PADDING;
        }
        vec3 delta = p1 - p0;
        vec3 delta_cross = normalize(cross(delta, vec_z));
        hit_miss = vec4(
            -1,  
            dot( delta_cross, d1u), 
            dot(-delta_cross, d0u),
            length(delta)
        );
    }
}"##);