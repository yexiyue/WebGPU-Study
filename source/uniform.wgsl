struct Params {
    color: vec4f,
    offset: vec2f,
    scale: f32,
}

@group(0) @binding(0) var<uniform> params: Params;

struct VertexOutput {
    @builtin(position) position: vec4f,
}

@vertex
fn vs(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var pos = array(
        vec2f(0.0, 0.5),
        vec2f(-0.5, -0.5),
        vec2f(0.5, -0.5),
    );

    var output = VertexOutput(
        vec4f(pos[vertex_index] * params.scale + params.offset, 0.0, 1.0),
    );

    return output;
}


@fragment
fn fs() -> @location(0) vec4f {
    return params.color;
}
