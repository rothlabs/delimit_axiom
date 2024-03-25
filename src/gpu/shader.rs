pub const PASS_VERTEX_SOURCE: &str = r##"#version 300 es
in vec4 position;
void main() {
    gl_Position = position;
}
"##;

pub const COPY_FRAGMENT_SOURCE: &str = r##"#version 300 es
precision highp float;
precision highp sampler2D;
uniform sampler2D uv_tex;
uniform ivec2 viewport_position;
out vec4 outColor;
void main() {
    ivec2 coord = ivec2(gl_FragCoord.x, gl_FragCoord.y) - viewport_position;
    outColor = texelFetch(uv_tex, coord, 0);
}
"##;