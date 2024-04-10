use const_format::concatcp;
use super::shader_parts::{
    FACET_CORE, FACET_PARTS, RAY_CORE, GET_RAY_DUAL, 
    GET_RAY_QUAD, HONE_CORE, HONE,
};

pub const DUAL_FROM_UVS_SOURCE: &str = concatcp!(r##"#version 300 es
precision highp float;
precision highp sampler2D;
precision highp isampler2D;
uniform sampler2D uv_tex;
layout(location=0) out vec3 point;
layout(location=1) out vec4 deriv_u;
layout(location=2) out vec4 deriv_v;
"##,
FACET_CORE,
"void main() {",
    FACET_PARTS, 
    r##"
    int facet_i = 0;
    vec2 uv = vec2(0., 0.);
    if(out_coord.y < pair_size.y){
        facet_i = texelFetch(pair_tex, out_coord, 0).r;
        uv = texelFetch(uv_tex, out_coord, 0).rg;
    }else{
        out_coord.y = out_coord.y - pair_size.y;
        facet_i = texelFetch(pair_tex, out_coord, 0).g;
        uv = texelFetch(uv_tex, out_coord, 0).ba;
    }
    float[9] rays = get_facet_rays(facet_i, uv);
    point   = vec3(rays[0], rays[1], rays[2]);
    deriv_u = vec4(rays[3], rays[4], rays[5], uv.x);
    deriv_v = vec4(rays[6], rays[7], rays[8], uv.y);
}
"##);

pub const QUAD_FROM_DUAL_SOURCE: &str = concatcp!(r##"#version 300 es
precision highp float;
precision highp sampler2D;
precision highp isampler2D;
"##,
FACET_CORE, HONE_CORE, RAY_CORE, 
"void main() {",
    FACET_PARTS, GET_RAY_DUAL, HONE,
"}");

pub const HONE_QUAD_SOURCE: &str = concatcp!(r##"#version 300 es
precision highp float;
precision highp sampler2D;
precision highp isampler2D;
"##,
FACET_CORE, RAY_CORE, HONE_CORE, 
"void main() {",
    FACET_PARTS, GET_RAY_QUAD, HONE,
"}");

pub const HIT_MISS_SOURCE: &str = concatcp!(r##"#version 300 es
precision highp float;
precision highp sampler2D;
precision highp isampler2D;
float tolerance = 0.005;
uniform sampler2D point_tex;
uniform sampler2D deriv_tex_u;
uniform sampler2D deriv_tex_v;
out vec4 hit_miss;
"##,
FACET_CORE, RAY_CORE, 
"void main() {",
    FACET_PARTS, GET_RAY_QUAD, 
    r##"
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
}
"##);

pub const HONE_TRACE_SOURCE: &str = concatcp!(r##"#version 300 es
precision highp float;
precision highp sampler2D;
precision highp isampler2D;
uniform isampler2D pair_tex;
uniform sampler2D point_tex;
uniform sampler2D uv_tex;
uniform sampler2D box_tex;
layout(location=0) out vec4 uvs;
layout(location=1) out vec4 box;
layout(location=2) out vec3 point;
"##,
FACET_CORE, RAY_CORE, 
"void main() {",
    FACET_PARTS, GET_RAY_DUAL, HONE, 
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
uniform sampler2D point_tex;
uniform sampler2D uv_tex;
uniform sampler2D box_tex;
layout(location=0) out vec4 uvs;
layout(location=1) out vec4 box;
layout(location=2) out vec4 uvDirs;
layout(location=3) out vec3 dir;
"##,
FACET_CORE, RAY_CORE, 
"void main() {",
    FACET_PARTS, GET_RAY_DUAL, 
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








// pub const HONE_SOURCE: &str = concatcp!(r##"#version 300 es
// precision highp float;
// precision highp sampler2D;
// precision highp isampler2D;
// uniform isampler2D pair_tex;
// uniform sampler2D uv_tex;
// uniform sampler2D point_tex;
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
                            //     point = (p0_b + p1_b) / 2.;
                            //     uv = uv0_b;
                            // }


// pub const CENTER_SOURCE: &str = r##"#version 300 es
// precision highp float;
// precision highp sampler2D;
// uniform sampler2D point_tex;
// uniform ivec2 viewport_position;
// out vec4 outColor;
// void main() {
//     ivec2 coord = ivec2(gl_FragCoord.x, gl_FragCoord.y) - viewport_position;
//     vec4 p0 = texelFetch(point_tex, coord, 0);
//     vec4 p1 = texelFetch(point_tex, coord + ivec2(0,1), 0);
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

