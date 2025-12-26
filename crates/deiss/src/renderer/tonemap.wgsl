@group(0) @binding(0)
var color_tex: texture_2d<f32>;

@group(0) @binding(1)
var color_sampler: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(color_tex, color_sampler, vertex.uv);
    return color;
}
