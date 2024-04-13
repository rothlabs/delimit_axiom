use const_format::concatcp;
use super::shader_parts::{
    HEADER, CORE_PARTS,
    FACET_CORE, FACET_PARTS, ARROW_CORE, ARROW_DUAL, 
    ARROW_IN_POS, ARROW_PALETTE, ARROW_IN, ARROW_OUT, HONE,
};

pub const INIT_HONE_PALETTE_SOURCE: &str = concatcp!(
HEADER, FACET_CORE, ARROW_OUT, r##"
uniform sampler2D uv_tex;
void main() {"##,
    CORE_PARTS, FACET_PARTS, 
    r##"
    int facet_index = 0;
    vec2 uv = vec2(0., 0.);
    ivec2 in_pos = out_pos;
    if(!(in_pos.x < pair_size.x)){
        in_pos.x = in_pos.x - pair_size.x; 
    }
    if(!(in_pos.x < pair_size.x)){
        in_pos.x = in_pos.x - pair_size.x; 
    }
    if(in_pos.y < pair_size.y){
        facet_index = texelFetch(pair_tex, in_pos, 0).r;
        uv = texelFetch(uv_tex, in_pos, 0).rg;
    }else{
        in_pos.y = in_pos.y - pair_size.y;
        facet_index = texelFetch(pair_tex, in_pos, 0).g;
        uv = texelFetch(uv_tex, in_pos, 0).ba;
    }
    output_arrows(facet_index, uv);
}
"##);

pub const HONE_PALETTE_SOURCE: &str = concatcp!(
HEADER, FACET_CORE, ARROW_CORE, ARROW_IN, ARROW_OUT,
"void main() {",
    CORE_PARTS, FACET_PARTS, ARROW_IN_POS, ARROW_PALETTE, HONE, r##"
}"##);

pub const HIT_MISS_SOURCE: &str = concatcp!(
HEADER, ARROW_IN, r##"
float tolerance = 0.005;
out vec4 hit_miss;
void main() {"##,
    CORE_PARTS, ARROW_IN_POS, ARROW_PALETTE, r##"
    float dist = length(p0 - p1);
    vec3 normal0 = -normalize(cross(d0u, d0v));
    vec3 normal1 = -normalize(cross(d1u, d1v));
    if(dist < tolerance){
        if(abs(dot(normal0, normal1)) < 0.995){     
            hit_miss = uvs;
        }else{
            hit_miss = vec4(-1, 0, 0, 0); 
        }
    }else{
        hit_miss = vec4(
            -1, 
            dist, 
            dot(normalize(p1 - p0), normal1), 
            dot(normalize(p0 - p1), normal0)
        );
    }
}"##);

pub const INIT_TRACE_PALETTE_SOURCE: &str = concatcp!(
HEADER, FACET_CORE, ARROW_OUT, r##"
uniform sampler2D uv_tex;
    //uniform sampler2D box_tex;
layout(location=3) out vec4 box;
void main() {"##,
    CORE_PARTS, FACET_PARTS, r##"
    int facet_index = 0;
    vec2 uv = vec2(0., 0.);
    ivec2 in_pos = out_pos;
    if(!(in_pos.x < pair_size.x)){
        in_pos.x = in_pos.x - pair_size.x; 
    }
    if(!(in_pos.x < pair_size.x)){
        in_pos.x = in_pos.x - pair_size.x; 
    }
    if(in_pos.y < pair_size.y){
        facet_index = texelFetch(pair_tex, in_pos, 0).r;
        uv = texelFetch(uv_tex, in_pos, 0).rg;
    }else{
        in_pos.y = in_pos.y - pair_size.y;
        facet_index = texelFetch(pair_tex, in_pos, 0).g;
        uv = texelFetch(uv_tex, in_pos, 0).ba;
    }
    output_arrows(facet_index, uv);
        //box = texelFetch(box_tex, in_pos, 0);
    box = vec4(1., 1., 0., 0.);
    box.x = min(box.x, uv.x);
    box.y = min(box.y, uv.y);
    box.z = max(box.z, uv.x);
    box.w = max(box.w, uv.y);
}
"##);

pub const TRACE_SEGMENT_SOURCE: &str = concatcp!( // TODO: does not need pair_tex, only size!
HEADER, ARROW_IN, r##"
// layout(location=0) out vec3 point;
// layout(location=1) out vec3 delta;
// layout(location=2) out vec4 arrow0;
// layout(location=3) out vec4 arrow1;
// uniform ivec2 viewport_position;
uniform int half_trace_count;
layout(location=0) out vec3 origin;
layout(location=1) out vec3 delta;
layout(location=2) out vec4 uvs_out;
layout(location=3) out vec4 uv_vectors;
void main() {"##,
    CORE_PARTS, r##"
            // int direction_tile_y = 0;
            // if(out_pos.y > 0){ // backward trace
            //     direction_tile_y = pair_size.y;
            // }
            // ivec2 in_pos0 = ivec2(out_pos.x % pair_size.x, (out_pos.x / pair_size.x) + direction_tile_y);
            // out_pos = out_pos - viewport_position;
    int y = out_pos.x / pair_size.x;
    ivec2 in_pos0a = ivec2(out_pos.x % pair_size.x,  y);
    ivec2 in_pos0b = ivec2(in_pos0a.x + pair_size.x, y);
    ivec2 in_pos0c = ivec2(in_pos0b.x + pair_size.x, y);
    y = y + pair_size.y;
    ivec2 in_pos1a = ivec2(in_pos0a.x, y);
    ivec2 in_pos1b = ivec2(in_pos0b.x, y);
    ivec2 in_pos1c = ivec2(in_pos0c.x, y);
    "##, ARROW_PALETTE, r##"
        // float sign = -1.;
        // if(out_pos.x < half_trace_count){
        //     sign = 1.;
        // }
    origin = (p0 + p1) / 2.;
    vec3 cross0 = cross(d0u, d0v);
    vec3 cross1 = cross(d1u, d1v);
    delta = normalize(cross(cross0, cross1));
            // arrow0  = vec4(t0u.a, t0v.a, dot(normalize(d0u), delta), dot(normalize(d0v), delta));
            // arrow1  = vec4(t1u.a, t1v.a, dot(normalize(d1u), delta), dot(normalize(d1v), delta));
    uvs_out = uvs;
    float du0 = dot(normalize(d0u), delta);// *100. / length(d0u);
    float dv0 = dot(normalize(d0v), delta);// *100. / length(d0v);
    float du1 = dot(normalize(d1u), delta);// *100. / length(d1u);
    float dv1 = dot(normalize(d1v), delta);// *100. / length(d1v);
    uv_vectors = vec4(du0, dv0, du1, dv1);
}"##);


pub const TRACE_DUAL_SOURCE: &str = concatcp!(
HEADER, FACET_CORE, ARROW_CORE, ARROW_IN, ARROW_OUT, r##"
float step = 0.8;
uniform int trace_count;
uniform sampler2D box_tex;
layout(location=3) out vec4 box;
void main() {"##,
    CORE_PARTS, FACET_PARTS, ARROW_IN_POS, ARROW_PALETTE, r##"
    int facet_index = 0;
    vec2 uv = vec2(0., 0.);
    vec3 du = vec3(0., 0., 0.);
    vec3 dv = vec3(0., 0., 0.);
    ivec2 box_in_pos = ivec2(in_pos0a.x, out_pos.y);
    if(out_pos.y < pair_size.y){
        facet_index = texelFetch(pair_tex, in_pos0a, 0).r;
        uv = uvs.rg; du = d0u; dv = d0v;
        if(pick > 0){
            box_in_pos.x = in_pos0b.x;
        }
    }else{
        facet_index = texelFetch(pair_tex, in_pos0a, 0).g;
        uv = uvs.ba; du = d1u; dv = d1v;
        if(pick < 1){
            box_in_pos.x = in_pos0b.x;
        }
    }
    if(pick > 1){
        box_in_pos.x = in_pos0c.x;
    }
    int trace_index = in_pos0a.y * pair_size.x + in_pos0a.x;
    float sign = -1.;
    if(trace_index < trace_count){
        sign = 1.;
    }
    vec3 cross0 = cross(d0u, d0v);
    vec3 cross1 = cross(d1u, d1v);
    vec3 delta = normalize(cross(cross0, cross1));
    delta = sign * delta * step;
    uv = get_uv_from_3d_delta(uv, du, dv, delta);
    output_arrows(facet_index, uv);
    box = texelFetch(box_tex, box_in_pos, 0);
}"##);

pub const TRACE_PALETTE_SOURCE: &str = concatcp!(
HEADER, FACET_CORE, ARROW_CORE, ARROW_IN, ARROW_OUT, r##"
uniform sampler2D box_tex;
layout(location=3) out vec4 box;
void main() {"##,
    CORE_PARTS, FACET_PARTS, ARROW_DUAL, HONE, r##"
    if(out_pos.y < pair_size.y){
        box = texelFetch(box_tex, in_pos0a, 0);
    }else{
        box = texelFetch(box_tex, in_pos1a, 0);
    }
    box.x = min(box.x, uv.x);
    box.y = min(box.y, uv.y);
    box.z = max(box.z, uv.x);
    box.w = max(box.w, uv.y);
}"##);

pub const BOXES_DUAL: &str = concatcp!(
HEADER, ARROW_IN, r##"
uniform sampler2D box_tex;
out vec4 box;
void main() {"##,
    CORE_PARTS, r##"
    int y = out_pos.x / pair_size.x;
    ivec2 in_pos0a = ivec2(out_pos.x % pair_size.x,  y);
    ivec2 in_pos0b = ivec2(in_pos0a.x + pair_size.x, y);
    ivec2 in_pos0c = ivec2(in_pos0b.x + pair_size.x, y);
    y = y + pair_size.y;
    ivec2 in_pos1a = ivec2(in_pos0a.x, y);
    ivec2 in_pos1b = ivec2(in_pos0b.x, y);
    ivec2 in_pos1c = ivec2(in_pos0c.x, y);
    "##, ARROW_PALETTE, r##"
    ivec2 box_in_pos = in_pos0a;
    if(pick > 1){
        box_in_pos = in_pos0c;
    }else if(pick > 0){
        box_in_pos = in_pos0b;
    }
    box = texelFetch(box_tex, box_in_pos, 0);
}"##);



////////////////////////////////////////////////////


pub const HONE_TRACE_SOURCE: &str = concatcp!(r##"#version 300 es
precision highp float;
precision highp sampler2D;
precision highp isampler2D;
uniform isampler2D pair_tex;
uniform sampler2D origin_tex;
uniform sampler2D uv_tex;
uniform sampler2D box_tex;
layout(location=0) out vec4 uvs;
layout(location=1) out vec4 box;
layout(location=2) out vec3 point;
"##,
FACET_CORE, ARROW_CORE, 
"void main() {",
    FACET_PARTS, ARROW_DUAL, HONE, 
    r##"
    box = texelFetch(box_tex, pair_coord, 0);
    vec2 uv = vec2(0, 0);
    if(i < 1){
        uvs = vec4(uv0_a.x, uv0_a.y, uv1_a.x, uv1_a.y);
        point = (p0_a + p1_a) / 2.;
        uv = uv0_a;
    // }else if(i < 2){
    //     uvs = vec4(uv0_b.x, uv0_b.y, uv1_b.x, uv1_b.y);
    //     point = (p0_b + p1_b) / 2.;
    //     uv = uv0_b;
    }else if(i < 2){
        uvs = vec4(uv0_c.x, uv0_c.y, uv1.x, uv1.y);
        point = (p0_c + p1a) / 2.;
        uv = uv0_c;
    }else{
        uvs = vec4(uv0.x, uv0.y, uv1_c.x, uv1_c.y);
        point = (p0a + p1_c) / 2.;
        uv = uv0;
    }
    box.x = min(box.x, uv.x);
    box.y = min(box.y, uv.y);
    box.z = max(box.z, uv.x);
    box.w = max(box.w, uv.y);
}
"##);


pub const TRACE_SOURCE: &str = concatcp!(r##"#version 300 es
precision highp float;
precision highp sampler2D;
precision highp isampler2D;

float step = 0.8;
float tolerance = 0.005;
uniform isampler2D pair_tex;
uniform sampler2D origin_tex;
uniform sampler2D uv_tex;
uniform sampler2D box_tex;
layout(location=0) out vec4 uvs;
layout(location=1) out vec4 box;
layout(location=2) out vec4 uvDirs;
layout(location=3) out vec3 dir;
"##,
FACET_CORE, ARROW_CORE, 
"void main() {",
    FACET_PARTS, ARROW_DUAL, 
    r##"
    float sign = -1.;
    if(pair_coord.x < pair_size.x/2){
        sign = 1.;
    }
    vec3 normal0 = get_facet_normal(uv0, p0a, p0b, p0c);
    vec3 normal1 = get_facet_normal(uv1, p1a, p1b, p1c);
    dir = normalize(cross(normal0, normal1));
    vec3 target = dir * sign * step;
    vec2 uv0a = get_uv_from_3d_delta(uv0, p0a, p0b, p0c, target);
    vec2 uv1a = get_uv_from_3d_delta(uv1, p1a, p1b, p1c, target);
    uvs = vec4(uv0a.x, uv0a.y, uv1a.x, uv1a.y);
    box = texelFetch(box_tex, pair_coord, 0);
    vec2 dirs0 = normalize(uv0a*100.0 - uv0*100.0);
    vec2 dirs1 = normalize(uv1a*100.0 - uv1*100.0);
    uvDirs = vec4(dirs0.x, dirs0.y, dirs1.x, dirs1.y);
}
"##);





// ivec2 box_in_pos = in_pos0a;
//     if(out_pos.x < trace_count){
//         if(pick > 0){
//             box_in_pos.x = in_pos0b.x;
//         }
//     }else{
//         if(pick < 1){
//             box_in_pos.x = in_pos1b.x;
//         }
//         box_in_pos.y = in_pos1b.y;
//     }
//     if(pick > 1){
//         box_in_pos.x = in_pos0c.x;
//     }


// ivec2 in_pos0a = ivec2(out_pos.x % pair_size.x,  out_pos.x / pair_size.x);
        // ivec2 box_in_pos = in_pos0a;
        // if(out_pos.x < trace_count){
        //     if(pick > 0){
        //         box_in_pos.x = in_pos0a.x + pair_size.x;
        //     }
        // }else{
        //     if(pick < 1){
        //         box_in_pos.x = in_pos0a.x + pair_size.x;
        //     }
        //     box_in_pos.y = in_pos0a.y + pair_size.y;
        // }
        // if(pick > 1){
        //     box_in_pos.x = in_pos0a.x + pair_size.x * 2;
        // }




// pub const HONE_SOURCE: &str = concatcp!(r##"#version 300 es
// precision highp float;
// precision highp sampler2D;
// precision highp isampler2D;
// uniform isampler2D pair_tex;
// uniform sampler2D uv_tex;
// uniform sampler2D origin_tex;
// out vec4 uvs;
// "##,
// FACET_CORE, UV_POINT_CORE, 
// "void main() {",
//     FACET_PARTS, UV_POINT_PARTS, HONE_PARTS, 
//     r##"
//     if(i < 1){
//         uvs = vec4(uv0_a.x, uv0_a.y, uv1_a.x, uv1_a.y);
//     // }else if(i < 2){
//     //     uvs = vec4(uv0_b.x, uv0_b.y, uv1_b.x, uv1_b.y);
//     }else if(i < 2){
//         uvs = vec4(uv0_c.x, uv0_c.y, uv1.x, uv1.y);
//     }else{
//         uvs = vec4(uv0.x, uv0.y, uv1_c.x, uv1_c.y);
//     }
// }
// "##);



// pub const POINT_SOURCE: &str = concatcp!(r##"#version 300 es
// precision highp float;
// precision highp sampler2D;
// precision highp isampler2D;
// uniform isampler2D pair_tex;
// uniform sampler2D uv_tex;
// out vec3 point;
// "##,
// FACET_CORE,
// "void main() {",
//     FACET_PARTS, 
//     r##"
//     int tile_x = 0;
//     if(pair_coord.x > pair_size.x-1){ 
//         pair_coord.x = pair_coord.x - pair_size.x; 
//         tile_x = 1;
//     }
//     if(pair_coord.x > pair_size.x-1){ 
//         pair_coord.x = pair_coord.x - pair_size.x; 
//         tile_x = 2;
//     }
//     int facet_i = 0;
//     vec2 uv = vec2(0., 0.);
//     if(pair_coord.y < pair_size.y){
//         facet_i = texelFetch(pair_tex, pair_coord, 0).r;
//         uv = texelFetch(uv_tex, pair_coord, 0).rg;
//     }else{
//         pair_coord.y = pair_coord.y - pair_size.y;
//         facet_i = texelFetch(pair_tex, pair_coord, 0).g;
//         uv = texelFetch(uv_tex, pair_coord, 0).ba;
//     }
//     if(tile_x == 0){
//         point = get_point_on_facet(facet_i, uv);
//     }else if(tile_x == 1){
//         if(uv.x > 0.5){uv = uv - vec2(uv_shift, 0.);}
//         else{uv = uv + vec2(uv_shift, 0.);}
//         point = get_point_on_facet(facet_i, uv);
//     }else if(tile_x == 2){
//         if(uv.y > 0.5){uv = uv - vec2(0., uv_shift);}
//         else{uv = uv + vec2(0., uv_shift);}
//         point = get_point_on_facet(facet_i, uv);
//     }
// }
// "##);




                            // if(length(p0_a - p1_a) < length(p0_b - p1_b)){
                            //     uvs = vec4(uv0_a.x, uv0_a.y, uv1_a.x, uv1_a.y);
                            //     point = (p0_a + p1_a) / 2.;
                            //     uv = uv0_a;
                            // }else{
                            //     uvs = vec4(uv0_b.x, uv0_b.y, uv1_b.x, uv1_b.y);
                            //     origin = (p0_b + p1_b) / 2.;
                            //     uv = uv0_b;
                            // }


// pub const CENTER_SOURCE: &str = r##"#version 300 es
// precision highp float;
// precision highp sampler2D;
// uniform sampler2D origin_tex;
// uniform ivec2 viewport_position;
// out vec4 outColor;
// void main() {
//     ivec2 coord = ivec2(gl_FragCoord.x, gl_FragCoord.y) - viewport_position;
//     vec4 p0 = texelFetch(origin_tex, coord, 0);
//     vec4 p1 = texelFetch(origin_tex, coord + ivec2(0,1), 0);
//     outColor = (p0 + p1) / 2.;
// }
// "##;

// pub const BOX_SOURCE: &str = r##"#version 300 es
// precision highp float;
// precision highp sampler2D;
// uniform sampler2D uv_tex;
// uniform sampler2D box_tex;
// out vec4 output0;
// void main() {
//     ivec2 box_tex_size = textureSize(box_tex, 0);
//     ivec2 box_coord = ivec2(gl_FragCoord.x, gl_FragCoord.y);
//     vec4 box = texelFetch(box_tex, box_coord, 0);

//     vec2 uv_f  = texelFetch(uv_tex, box_coord, 0).rg;
//     box.x = min(box.x, uv_f.x);
//     box.y = min(box.y, uv_f.y);
//     box.z = max(box.z, uv_f.x);
//     box.w = max(box.w, uv_f.y);

//     ivec2 box_coord_r = ivec2(int(gl_FragCoord.x) + box_tex_size.x, gl_FragCoord.y);
//     vec2 uv_r  = texelFetch(uv_tex, box_coord_r, 0).rg;
//     output0.x = min(box.x, uv_r.x);
//     output0.y = min(box.y, uv_r.y);
//     output0.z = max(box.z, uv_r.x);
//     output0.w = max(box.w, uv_r.y);
// }
// "##;

