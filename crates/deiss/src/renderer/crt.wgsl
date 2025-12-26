@group(0) @binding(0)
var paint_tex: texture_2d<f32>;

@group(0) @binding(1)
var paint_sampler: sampler;

@group(0) @binding(2)
var prev_tex: texture_2d<f32>;

@group(0) @binding(3)
var prev_sampler: sampler;

struct CrtSettings {
    paint_shape: vec2<f32>,
    display_shape: vec2<f32>,
    warp: vec2<f32>,
    scan: f32,
    afterglow: f32,
}

@group(0) @binding(4)
var<uniform> settings: CrtSettings;

/// Map UV from display (high-res) to painted texture (640x480)
fn adjust_uv(uv: vec2<f32>) -> vec2<f32> {
    let image_width = settings.paint_shape.y;
    let image_height = settings.paint_shape.x;
    let surface_width = settings.display_shape.y;
    let surface_height = settings.display_shape.x;

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
    return (uv - offset) / scale;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    // map from screen to painting
    var uv = adjust_uv(vertex.uv);

    // Check if we're outside the image bounds (in the black bars)
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return vec4(0.0, 0.0, 0.0, 1.0); // Black bars
    }

    // warp
    let dc = pow(abs(0.5 - uv), vec2(2.));
    uv.x -= 0.5; uv.x *= 1. + dc.y * settings.warp.x; uv.x += 0.5;
    uv.y -= 0.5; uv.y *= 1. + dc.x * settings.warp.y; uv.y += 0.5;

    // scanlines
    let image_height = settings.paint_shape.x;
    let scan = abs(sin(uv.y*3.1415*image_height)*0.5*settings.scan);

    let color = mix(textureSample(paint_tex, paint_sampler, uv).rgb, vec3(0.), scan);

    // previous value (output space)
    let prev = textureSample(prev_tex, prev_sampler, vertex.uv);

    // afterglow

    let glow = settings.afterglow * prev.rgb;
    let glow_len = length(glow);

    let delta = normalize(pow(glow + vec3(0.001), vec3(0.1))) * glow_len;

    var w = 1.;
    if (color.r + color.g + color.b > 7./255.) { w = 0.; }

    let out = color + w * delta;

    return vec4(out, 1.);
}
