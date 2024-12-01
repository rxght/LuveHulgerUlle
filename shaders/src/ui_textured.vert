#version 450

layout(location = 0) in vec2 pos;

layout(location = 0) out vec2 out_uv;

layout( set = 0, binding = 0 ) uniform LayoutData {
    vec2 position;
    vec2 dimensions;
};

void main()
{
    gl_Position = vec4(position + pos * dimensions, 0.0, 1.0);
    out_uv = pos;
}