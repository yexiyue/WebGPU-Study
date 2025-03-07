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

  // 创建绑定组布局
  const bindGroupLayout = device.createBindGroupLayout({
    label: "bind-group-layout",
    entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
          hasDynamicOffset: false,
          minBindingSize: 0,
        },
      },
    ],
  });

  // 创建绑定组
  const bindGroup = device.createBindGroup({
    layout: bindGroupLayout,
    entries: [
      {
        binding: 0,
        resource: {
          buffer: storageBuffer,
        },
      },
    ],
  });

  // 创建计算管道布局
  const computePipelineLayout = device.createPipelineLayout({
    label: "compute-pipeline-layout",
    bindGroupLayouts: [bindGroupLayout],
  });

  // 创建计算管道
  const computePipeline = device.createComputePipeline({
    label: "compute-pipeline",
    layout: computePipelineLayout,
    compute: {
      module: shaderModel,
      entryPoint: "main",
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
  const commandBuffer = encoder?.finish();
  device?.queue.submit([commandBuffer!]);

  // 映射结果缓冲区并读取数据
  await resultBuffer.mapAsync(GPUMapMode.READ);
  const result = new Float32Array(resultBuffer.getMappedRange());
  console.log("input", input);
  console.log("result", result);
  resultBuffer?.unmap();
}

// 调用主函数
main();