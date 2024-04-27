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
    vec3 deltaX = vec3(0., 0., 0.);
    if(out_pos.y < pair_size.y){
        curve_index = texelFetch(pair_tex, in_pos0a, 0).r;
        u = us.r; du = d0u; 
        pa = p0; pb = p1;
    }else{
        curve_index = texelFetch(pair_tex, in_pos0a, 0).g;
        u = us.g; du = d1u; 
        pa = p1; pb = p0;
    }
    if(out_pos.x < pair_size.x){
        point = pa;
        delta = vec4(du.x, du.y, du.z, u);
    }else{
        if(out_pos.x < pair_size.x * 2){
            deltaX = pb - pa;
        }else{
            deltaX = get_arrow_hit(p0, d0u, p1, d1u) - pa;
        }
        u = get_moved_u(u, du, deltaX);
        output_arrow(curve_index, u);
    }
}"##);

pub const HIT_MISS_SOURCE: &str = concatcp!(
HEADER, GEOM_CORE, ARROW_IN, r##"
float tolerance = 0.05;
vec3 vec_z = vec3(0., 0., 1.);
layout(location=0) out vec4 hit_miss;
layout(location=1) out vec3 point;
void main() {"##,
    CORE_PARTS, GEOM_PARTS, PALETTE_IN_POS, ARROW_PALETTE, r##"
    d0u = normalize(d0u);
    d1u = normalize(d1u);
    if(length(p0 - p1) < tolerance){
        // hit_miss = vec4(-10., -10., -10., -10.); 
        // if((us.r > 0.9999 && us.g < 0.0001) || (us.r < 0.0001 && us.g > 0.9999)){
        //     return;
        // }
        // if(abs(dot(d0u, d1u)) > 0.9999){     
        //     return;
        // }
        vec3 cross0 = normalize(cross(d0u, vec_z));
        vec3 cross1 = normalize(cross(d1u, vec_z));
        hit_miss = vec4(
            us.r,
            us.g,
            dot(cross0, d1u), 
            dot(cross1, d0u)
        );
        point = (p0 + p1) / 2.;
    }else{
        if(us.r < 0.0001){
            p0 = p0 + d0u * 0.0001;
        }else if(us.r > 0.9999){
            p0 = p0 - d0u * 0.0001;
        }
        if(us.g < 0.0001){
            p1 = p1 + d1u * 0.0001;
        }else if(us.g > 0.9999){
            p1 = p1 - d1u * 0.0001;
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