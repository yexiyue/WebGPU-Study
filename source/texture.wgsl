// 定义顶点输出结构体，包含位置和纹理坐标
struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) texcoord: vec2f,
}

// 定义绑定组和绑定点的变量，用于采样器和纹理
@group(0) @binding(0) var ourSampler: sampler;
@group(0) @binding(1) var ourTexture: texture_2d<f32>;

// 定义uniform变量，用于传递缩放参数
// 这里的缩放参数是一个二维向量，表示在x和y方向上的缩放比例
@group(1) @binding(0) var<uniform> scale: vec2f;

// 顶点着色器函数，计算顶点位置和纹理坐标
@vertex
fn vs(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // 定义顶点位置数组
    let pos = array(vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(-1.0, 1.0), vec2(1.0, -1.0), vec2(1.0, 1.0), vec2(-1.0, 1.0));

    // 返回顶点输出，位置经过缩放，纹理坐标归一化到[0, 1]
    return VertexOutput(vec4f(pos[vertex_index] * scale, 0.0, 1.0),(pos[vertex_index] + vec2(1.0, 1.0)) * 0.5);
}

// 片段着色器函数，采样纹理并返回颜色
@fragment
fn fs(in: VertexOutput) -> @location(0) vec4f {
    return textureSample(ourTexture, ourSampler, in.texcoord);
}
