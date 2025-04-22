struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) texcoord: vec2f,
}

@group(0) @binding(0) var ourSampler: sampler;
@group(0) @binding(1) var ourTexture: texture_2d<f32>;
@group(1) @binding(0) var<uniform> scale: vec2f;

@vertex
fn vs(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let pos = array(vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(-1.0, 1.0), vec2(1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0));

    return VertexOutput(vec4f(pos[vertex_index] * scale, 0.0, 1.0),(pos[vertex_index] + vec2(1.0, 1.0)) * 0.5);
}

@fragment
fn fs(in: VertexOutput) -> @location(0) vec4f {
    return textureSample(ourTexture, ourSampler, in.texcoord);
}
