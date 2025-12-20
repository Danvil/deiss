/// Applies ACES tone map for every fragment

@group(0) @binding(0)
var color_tex: texture_2d<f32>;

@group(0) @binding(1)
var color_sampler: sampler;

// Luminance of an sRGB color
fn luma_linear(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3(0.2126, 0.7152, 0.0722));
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(color_tex, color_sampler, vertex.uv);
    // let ace = aces_tonemap(color.rgb);
    // let luma = luma_linear(ace);
    // return vec4(ace, luma);
    return color;
}
