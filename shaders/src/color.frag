#version 450

layout( set = 1, binding = 0 ) uniform ColorData {
    vec4 color;
};

layout( location = 0 ) out vec4 out_color;

void main()
{
    out_color = color;
}