#version 450

layout(location = 0) in vec2 pos;

layout( set = 0, binding = 0 ) uniform CartesianToNormalizedUbo {
    mat4 cartesian_to_normalized;
};

layout( set = 2, binding = 0) uniform CameraUbo {
    mat4 camera;
};

void main()
{
    gl_Position =  cartesian_to_normalized * camera * vec4(pos, 0.0f, 1.0f);
}