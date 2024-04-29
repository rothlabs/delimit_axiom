use const_format::concatcp;

pub const ARROW_IN: &str = r##"
uniform sampler2D point_tex;
uniform sampler2D delta_tex;
uniform sampler2D param_tex;
"##;

pub const ARROW_OUT: &str = r##"
layout(location=0) out vec4 point;
layout(location=1) out vec4 delta;
layout(location=2) out vec2 param;
void output_arrow(int ci, float u){
    float[6] arrow = get_curve_arrow(ci, u);
        //float x_round = round(arrow[0] * 1000000.) / 1000000.;
        //float y_round = round(arrow[1] * 1000000.) / 1000000.;
        //point = vec4(x_round, y_round, arrow[0] - x_round, arrow[1] - y_round);
    point = vec4(arrow[0], arrow[1], 0., 0.);
        //float d0_round = round(arrow[3] * 1000000.) / 1000000.;
        //float d1_round = round(arrow[4] * 1000000.) / 1000000.;
        //delta = vec4(d0_round, d1_round, arrow[3] - d0_round, arrow[4] - d1_round);
    delta = vec4(arrow[3], arrow[4], 0., 0.);
        //float u_round = round(u * 1000000.) / 1000000.;
        //param = vec2(u, u - u_round);
    param = vec2(u, 0.);
}
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
    float u0 = 0.;
    float u1 = 0.;
            // vec4 t0 = vec4(0., 0., 0., 0.);
            // vec4 t1 = vec4(0., 0., 0., 0.);
    vec2 t0 = vec2(0., 0.);
    vec2 t1 = vec2(0., 0.);
            // vec3 d0u = vec2(0., 0.);
            // vec3 d1u = vec2(0., 0.);
    if(pick > 1){
        u0 = texelFetch(param_tex, in_pos0c,  0).r; // + texelFetch(param_tex, in_pos0c,  0).g;
        u1 = texelFetch(param_tex, in_pos1c,  0).r; // + texelFetch(param_tex, in_pos1c,  0).g;
        t0 = texelFetch(delta_tex, in_pos0c,  0).rg; // + texelFetch(delta_tex, in_pos0c,  0).ba;
        t1 = texelFetch(delta_tex, in_pos1c,  0).rg; // + texelFetch(delta_tex, in_pos1c,  0).ba;
    }else if(pick > 0){
        u0 = texelFetch(param_tex, in_pos0b,  0).r; // + texelFetch(param_tex, in_pos0b,  0).g;
        u1 = texelFetch(param_tex, in_pos1a,  0).r; // + texelFetch(param_tex, in_pos1a,  0).g;
        t0 = texelFetch(delta_tex, in_pos0b,  0).rg; // + texelFetch(delta_tex, in_pos0b,  0).ba;
        t1 = texelFetch(delta_tex, in_pos1a,  0).rg; // + texelFetch(delta_tex, in_pos1a,  0).ba;
    }else{
        u0 = texelFetch(param_tex, in_pos0a,  0).r; // + texelFetch(param_tex, in_pos0a,  0).g;
        u1 = texelFetch(param_tex, in_pos1b,  0).r; // + texelFetch(param_tex, in_pos1b,  0).g;
        t0 = texelFetch(delta_tex, in_pos0a,  0).rg; // + texelFetch(delta_tex, in_pos0a,  0).ba;
        t1 = texelFetch(delta_tex, in_pos1b,  0).rg; // + texelFetch(delta_tex, in_pos1b,  0).ba;
    }
            // vec3 d0u = t0.xyz;
            // vec3 d1u = t1.xyz;
    vec3 d0u = vec3(t0.x, t0.y, 0.);
    vec3 d1u = vec3(t1.x, t1.y, 0.);
"##;

pub const MOVE_U: &str = r##"
float get_moved_u(float u, vec3 du, vec3 p0, vec3 p1) {
        // if(isnan(delta.x) || isnan(delta.y) || isnan(delta.z) || length(delta) < 0.00001){  // 0.0001
        //     return u;
        // }
    float du_lg = length(du);
    float du_sq = du_lg * du_lg;
    float u0 = dot(p0, du) / du_sq;
    float u1 = dot(p1, du) / du_sq;
    u = clamp(u + u1 - u0, 0., 1.);
    return u;
}
"##;

// pub const MOVE_U: &str = r##"
// float get_moved_u(float u, vec3 du, vec3 delta) {
//     if(isnan(delta.x) || isnan(delta.y) || isnan(delta.z) || length(delta) < 0.0001){  
//         return u;
//     }
//     u = u + dot(normalize(du), normalize(delta)) / length(du) * length(delta);
//     u = clamp(u, 0., 1.);
//     return u;
// }
// "##;