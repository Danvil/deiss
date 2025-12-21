/// Applies ACES tone map for every fragment

@group(0) @binding(0)
var color_tex: texture_2d<f32>;

@group(0) @binding(1)
var color_sampler: sampler;

@group(0) @binding(2)
var<uniform> aspect_data: vec4<f32>; // image_width, image_height, surface_width, surface_height

// Luminance of an sRGB color
fn luma_linear(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3(0.2126, 0.7152, 0.0722));
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let image_width = aspect_data.x;
    let image_height = aspect_data.y;
    let surface_width = aspect_data.z;
    let surface_height = aspect_data.w;

    // Calculate aspect ratios
    let image_aspect = image_width / image_height;
    let surface_aspect = surface_width / surface_height;

    // Calculate the scale and offset for letterboxing/pillarboxing
    var scale: vec2<f32>;
    var offset: vec2<f32>;

    if (image_aspect > surface_aspect) {
        // Image is wider than surface - pillarbox (black bars on top/bottom)
        scale = vec2(1.0, surface_aspect / image_aspect);
        offset = vec2(0.0, (1.0 - scale.y) * 0.5);
    } else {
        // Image is taller than surface - letterbox (black bars on left/right)
        scale = vec2(image_aspect / surface_aspect, 1.0);
        offset = vec2((1.0 - scale.x) * 0.5, 0.0);
    }

    // Transform UV coordinates to maintain aspect ratio
    let adjusted_uv = (vertex.uv - offset) / scale;

    // Check if we're outside the image bounds (in the black bars)
    if (adjusted_uv.x < 0.0 || adjusted_uv.x > 1.0 || adjusted_uv.y < 0.0 || adjusted_uv.y > 1.0) {
        return vec4(0.0, 0.0, 0.0, 1.0); // Black bars
    }

    let color = textureSample(color_tex, color_sampler, adjusted_uv);
    // let ace = aces_tonemap(color.rgb);
    // let luma = luma_linear(ace);
    // return vec4(ace, luma);
    return color;
}
