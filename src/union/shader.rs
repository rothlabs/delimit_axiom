pub const CENTER_SHADER_SOURCE: &str = r##"#version 300 es
precision highp float;
precision highp sampler2D;
uniform sampler2D point_tex;
uniform ivec2 viewport_position;
out vec4 outColor;
void main() {
    ivec2 coord = ivec2(gl_FragCoord.x, gl_FragCoord.y) - viewport_position;
    vec4 p0 = texelFetch(point_tex, coord, 0);
    vec4 p1 = texelFetch(point_tex, coord + ivec2(0,1), 0);
    outColor = (p0 + p1) / 2.;
}
"##;