#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 out_uv;

layout( set = 0, binding = 0 ) uniform CartesianToNormalizedUbo {
    mat4 cartesian_to_normalized;
};

layout( set = 2, binding = 0) uniform CameraUbo {
    mat4 camera;
};

layout( set = 3, binding = 0) uniform FrameData {
    vec2 uv_offset;
};

void main()
{
    gl_Position =  cartesian_to_normalized * camera * vec4(pos, 0.0f, 1.0f);
    out_uv = uv + uv_offset;
}