// 定义一个存储缓冲区，绑定在组0，绑定点0
@group(0) @binding(0) var<storage, read_write> data: array<f32>;

// 定义一个计算着色器，工作组大小为1
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3u) {
    // 将数据中的每个元素乘以2
    data[id.x] *= 2.0;
}