import "./style.css";
import computeWgsl from "../../source/compute.wgsl?raw";

async function main() {
  // 请求 GPU 适配器
  const adapter = await navigator?.gpu.requestAdapter({
    powerPreference: "low-power",
  });

  // 请求 GPU 设备
  const device = await adapter?.requestDevice();
  if (!device) {
    throw new Error("Failed to create device");
  }

  // 输入数据
  const input = new Float32Array([1, 2, 3, 4]);

  // 创建存储缓冲区
  const storageBuffer = device.createBuffer({
    label: "storage-buffer",
    size: input.byteLength,
    usage:
      GPUBufferUsage.STORAGE |
      GPUBufferUsage.COPY_DST |
      GPUBufferUsage.COPY_SRC,
  });

  // 将输入数据写入存储缓冲区
  device?.queue.writeBuffer(storageBuffer, 0, input.buffer);

  // 创建结果缓冲区
  const resultBuffer = device?.createBuffer({
    label: "result-buffer",
    size: storageBuffer.size,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
  });

  // 创建着色器模块
  const shaderModel = device.createShaderModule({
    label: "shader-model",
    code: computeWgsl,
  });

  // 创建绑定组布局时，通过设备(device)调用createBindGroupLayout方法，并传入包含绑定信息的对象。
  const bindGroupLayout = device.createBindGroupLayout({
    label: "bind-group-layout", // 绑定组布局的标识符
    entries: [
      {
        binding: 0, // 绑定点索引
        visibility: GPUShaderStage.COMPUTE, // 指定该资源对计算着色器阶段可见
        buffer: {
          // 资源类型为存储缓冲区
          type: "storage",
          hasDynamicOffset: false, // 是否支持动态偏移
          minBindingSize: 0, // 最小绑定大小
        },
      },
    ],
  });

  // 创建绑定组时，根据之前定义的绑定组布局(bindGroupLayout)，通过device.createBindGroup方法将实际的存储缓冲区(storageBuffer)绑定到0号位。
  const bindGroup = device.createBindGroup({
    layout: bindGroupLayout, // 使用的绑定组布局
    entries: [
      {
        binding: 0,
        resource: {
          // 实际绑定的资源
          buffer: storageBuffer, // 存储缓冲区
        },
      },
    ],
  });

  // 创建计算管道布局时，通过设备(device)调用createPipelineLayout方法，并传入包含绑定组布局(bindGroupLayouts)的数组。
  const computePipelineLayout = device.createPipelineLayout({
    label: "compute-pipeline-layout", // 计算管道布局的标识符
    bindGroupLayouts: [bindGroupLayout], // 使用的绑定组布局列表
  });

  // 创建计算管道时，根据之前定义的计算管道布局(computePipelineLayout)，通过device.createComputePipeline方法指定计算着色器模块(shaderModel)及其入口点(entryPoint)。
  const computePipeline = device.createComputePipeline({
    label: "compute-pipeline", // 计算管道的标识符
    layout: computePipelineLayout, // 使用的计算管道布局
    compute: {
      module: shaderModel, // 计算着色器模块
      entryPoint: "main", // 着色器程序入口点
    },
  });

  // 创建命令编码器
  const encoder = device.createCommandEncoder({
    label: "compute-encoder",
  });
  const pass = encoder.beginComputePass({
    label: "compute-pass",
  });
  pass.setPipeline(computePipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(input.length);
  pass.end();

  // 将存储缓冲区的数据复制到结果缓冲区
  encoder.copyBufferToBuffer(
    storageBuffer,
    0,
    resultBuffer,
    0,
    input.byteLength
  );

  // 提交命令缓冲区
  const commandBuffer = encoder.finish();
  device.queue.submit([commandBuffer]);

  // 映射结果缓冲区并读取数据
  await resultBuffer.mapAsync(GPUMapMode.READ);
  const result = new Float32Array(resultBuffer.getMappedRange());
  console.log("input", input);
  console.log("result", result);
  resultBuffer?.unmap();
}

// 调用主函数
main();
