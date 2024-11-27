#version 450

layout(location = 0) in vec2 pos;

layout(location = 0) out vec3 out_uv;

layout( set = 0, binding = 0 ) uniform CartesianToNormalizedUbo {
    mat4 cartesian_to_normalized;
};

layout( set = 2, binding = 0) uniform CameraUbo {
    mat4 camera;
};

layout( push_constant ) uniform ObjectData {
    vec2 position;
    vec2 dimensions;
    float layer_idx;
};

void main()
{
    vec2 vertex_pos = vec2(pos.x, -pos.y) * dimensions;
    gl_Position =  cartesian_to_normalized * camera * vec4(vertex_pos + position, 0.0f, 1.0f);
    out_uv = vec3(pos, layer_idx);
}