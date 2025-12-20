struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Draws one large triangle over the clip space when called with three vertices
//
//  -1       0       1       2       3
//    +--------------+................
//    |              |              .
//    |              |           .
//  0 |              |        .
//    |              |      .
//    |              |    .
//    |              | .
//  1 +--------------+
//    .            .
//    .          .
//  2 .       .
//    .     .
//    .   .
//  3 ..
//
// Must be called with 3 vertex indices: 0, 1, 2.
//
// Reference:
// - Vulkan tutorial on rendering a fullscreen quad without buffers, Sascha Willems
//   https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // vertex indices to triangle corners
    let tc = vec2(
        f32(vertex_index & 2u),        // 0, 0, 2
        f32((vertex_index & 1u) << 1)  // 0, 2, 0
    );

    // Map from [0,2] to [-1,3] so that the the rectangle inscribed in the triangle covers the NDC
    // range [-1,+1]
    let pos = vec4(
        tc.x * 2.0 - 1.0,
        1.0 - tc.y * 2.0,
        0.0,
        1.0
    );

    // UV coordinates run over [0,1] for [-1,+1] pos
    let uv = tc;

    return VertexOutput(pos, uv);
}
