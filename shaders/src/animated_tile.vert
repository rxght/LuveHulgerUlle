#version 450
//#extension GL_EXT_debug_printf : enable

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 out_uv;

layout( set = 0, binding = 0 ) uniform CartesianToNormalizedUbo {
    mat4 cartesian_to_normalized;
};

layout( set = 2, binding = 0) uniform CameraUbo {
    mat4 camera;
};

layout( push_constant ) uniform ObjectData {
    vec2 object_position;
    vec2 base_uv_offset;
    float frame_uv_stride;
    uint frame_offset;
};

void main()
{
    gl_Position = cartesian_to_normalized * camera * vec4(pos + object_position, 0.0f, 1.0f);
    //debugPrintfEXT("uv_x_offset = %f\n", frame_uv_stride * frame_offset);
    out_uv = uv + base_uv_offset + vec2(frame_uv_stride * frame_offset, 0.0);
}