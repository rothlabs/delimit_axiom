pub const PASS_VERTEX_SOURCE: &str = r##"#version 300 es
in vec4 position;
void main() {
    gl_Position = position;
}
"##;

pub const COPY_FRAGMENT_SOURCE: &str = r##"#version 300 es
precision highp float;
precision highp sampler2D;
uniform ivec2 viewport_position;
uniform sampler2D source_tex0;
uniform sampler2D source_tex1;
uniform sampler2D source_tex2;
layout(location=0) out vec4 output0;
layout(location=1) out vec4 output1;
layout(location=2) out vec4 output2;
void main() {
    ivec2 coord = ivec2(gl_FragCoord.x, gl_FragCoord.y) - viewport_position;
    output0 = texelFetch(source_tex0, coord, 0);
    output1 = texelFetch(source_tex1, coord, 0);
    output2 = texelFetch(source_tex2, coord, 0);
}
"##;