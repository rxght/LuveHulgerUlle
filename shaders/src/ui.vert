#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec3 uv;

layout(location = 0) out vec3 out_uv;

layout( set = 0, binding = 0 ) uniform CartesianToNormalizedUbo {
    mat4 cartesian_to_normalized;
};

void main()
{
    gl_Position =  cartesian_to_normalized * vec4(pos.x, -pos.y, 0.0f, 1.0f);
    out_uv = uv;
}