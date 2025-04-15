// 为每个字段添加@location属性
struct Params {
    @location(1) color: vec4f,
    @location(2) offset: vec2f,
    @location(3) scale: f32,
}

@group(0) @binding(0) var<storage> params_list: array<Params>;

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
}

// 顶点输入结构体
// 这里我们只需要位置数据
struct VertexInput {
    @location(0) position: vec2f,
}

@vertex
fn vs(
    // @builtin(vertex_index) vertex_index: u32 // 这里我们不需要使用内置的顶点索引
    vertex_input: VertexInput, 
    // @builtin(instance_index) instance_index: u32
    params: Params
) -> VertexOutput {

    // 使用instance_index来选择params_list中的参数
    // let params = params_list[instance_index];

    // var pos = array(
    //     vec2f(0.0, 0.5),
    //     vec2f(-0.5, -0.5),
    //     vec2f(0.5, -0.5),
    // );

    // 使用vertex_input.position来获取顶点位置
    var output = VertexOutput(
        vec4f(vertex_input.position * params.scale + params.offset, 0.0, 1.0),
        params.color,
    );

    return output;
}


@fragment
fn fs(vsOutput: VertexOutput) -> @location(0) vec4f {
    return vsOutput.color;
}
