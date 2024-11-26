#version 450

layout(location = 0) in vec2 pos;

layout(location = 0) out vec3 out_uv;

layout( set = 0, binding = 0 ) uniform CartesianToNormalizedUbo {
    mat4 cartesian_to_normalized;
};

layout( set = 2, binding = 0) uniform CameraUbo {
    mat4 camera;
};

layout( push_constant ) uniform FrameData {
    float frame_idx;
    float tile_width;
    float tile_height;
};

void main()
{
    gl_Position =  cartesian_to_normalized * camera * vec4(pos * vec2(tile_width, -tile_height), 0.0f, 1.0f);
    out_uv = vec3(pos, frame_idx);
}