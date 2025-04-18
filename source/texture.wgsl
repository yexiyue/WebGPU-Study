struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) texcoord: vec2f,
}

@group(0) @binding(0) var ourSampler: sampler;
@group(0) @binding(1) var ourTexture: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let pos = array(
        // 1st triangle
        vec2f(0.0, 0.0), // center
        vec2f(1.0, 0.0), // right, center
        vec2f(0.0, 1.0), // center, top
        // 2st triangle
        vec2f(0.0, 1.0), // center, top
        vec2f(1.0, 0.0), // right, center
        vec2f(1.0, 1.0), // right, top
    );

    return VertexOutput(vec4f(pos[vertex_index], 0.0, 1.0), pos[vertex_index]);
}

@fragment
fn fs(in: VertexOutput) -> @location(0) vec4f {
    return textureSample(ourTexture, ourSampler, in.texcoord);
}
