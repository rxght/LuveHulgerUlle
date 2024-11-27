#version 450

layout(location = 0) in vec2 pos;
layout(location = 1) in vec3 uv;

layout(location = 0) out vec3 out_uv;

layout( set = 0, binding = 0 ) uniform CartesianToNormalizedUbo {
    mat4 cartesian_to_normalized;
};

layout( set = 2, binding = 0) uniform CameraUbo {
    mat4 camera;
};

layout( push_constant ) uniform FrameData {
    float layer_offset;
};

void main()
{
    gl_Position =  cartesian_to_normalized * camera * vec4(pos.x, -pos.y, 0.0f, 1.0f);
    out_uv = vec3(uv.xy, uv.z + layer_offset);
}