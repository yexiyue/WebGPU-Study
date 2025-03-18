@vertex
fn vs(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    var pos = array(vec2f(0.0, 0.5), // 顶点1（顶部中心）
    vec2f(-0.5, -0.5), // 顶点1（顶部中心）
    vec2f(0.5, -0.5) // 顶点1（顶部中心）
    );
    return vec4f(pos[vertex_index], 0.0, 1.0); // 扩展为4D向量
}

@fragment
fn fs() -> @location(0) vec4f {
    return vec4f(1.0, 0.0, 0.0, 1.0); // 纯红色（不透明）
}
