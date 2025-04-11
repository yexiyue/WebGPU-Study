struct Params {
    color: vec4f,
    offset: vec2f,
    scale: f32,
}

// 将原来的params改为params_list，并将其声明为数组
@group(0) @binding(0) var<storage> params_list: array<Params>;

struct VertexOutput {
    @builtin(position) position: vec4f,
    // 新增color属性，片段着色器中访问不到 @builtin(instance_index)
    @location(0) color: vec4f,
}

@vertex
fn vs(@builtin(vertex_index) vertex_index: u32,
// 新增参数，表示实例索引
// 这里的instance_index是一个内置变量，表示当前实例的索引
@builtin(instance_index) instance_index: u32) -> VertexOutput {
    var pos = array(
        vec2f(0.0, 0.5),
        vec2f(-0.5, -0.5),
        vec2f(0.5, -0.5),
    );

    // 使用instance_index来选择params_list中的参数
    let params = params_list[instance_index];

    var output = VertexOutput(
        vec4f(pos[vertex_index] * params.scale + params.offset, 0.0, 1.0),
        params.color,
    );

    return output;
}


@fragment
fn fs(vsOutput: VertexOutput) -> @location(0) vec4f {
    return vsOutput.color;
}
