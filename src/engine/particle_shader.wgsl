struct VertexOutput
{
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv : vec2<f32>
}

struct VertexInput
{
    @location(0) position : vec3<f32>,
    @location(1) uv : vec2<f32>,
}

struct CameraUniform {
    proj_view: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(5) offset: vec2<f32>,
};


@vertex
fn vs_main(
    model : VertexInput,
    instance : InstanceInput
) -> VertexOutput
{
    var out : VertexOutput;
    
    let offset = vec2<f32>(0.5, 0.288675134595);

    out.clip_position = camera.proj_view * vec4<f32>(model.position.xy + instance.offset - offset, model.position.z, 1.0);
    out.uv = model.uv;
    return out;
}


@group(0) @binding(0)
var texture : texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler : sampler;

@fragment 
fn fs_main(in : VertexOutput) -> @location(0) vec4<f32> {
    //return vec4<f32>(in.uv.xy, 0., 1.);
    return textureSample(texture, texture_sampler, in.uv); 
}