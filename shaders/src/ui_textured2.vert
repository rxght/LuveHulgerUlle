#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 out_uv;

layout( set = 0, binding = 0 ) uniform CartesianToNormalizedUbo {
    mat4 cartesian_to_normalized;
};

layout( push_constant ) uniform PCR {
    float ui_scale;
};

void main()
{
    vec4 scaled_pos = vec4(ui_scale * pos.x, ui_scale * -pos.y, 0.0f, 1.0f);
    gl_Position = cartesian_to_normalized * scaled_pos;
    out_uv = uv;
}