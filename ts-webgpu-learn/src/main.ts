import "./style.css";
import uniformWgsl from "../../source/uniform.wgsl?raw";
import { makeShaderDataDefinitions, makeStructuredView } from "webgpu-utils";

class WebGPUApp {
  constructor(
    public device: GPUDevice,
    public queue: GPUQueue,
    public canvas: HTMLCanvasElement,
    public ctx: GPUCanvasContext,
    public pipeline: GPURenderPipeline,
    public bindGroup: GPUBindGroup
  ) {
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const canvas = entry.target as HTMLCanvasElement;
        const width = entry.contentBoxSize[0].inlineSize;
        const height = entry.contentBoxSize[0].blockSize;
        canvas.width = Math.min(width, device.limits.maxTextureDimension2D);
        canvas.height = Math.min(height, device.limits.maxTextureDimension2D);
      }
    });
    observer.observe(canvas);
  }

  public static async create() {
    const adapter = await navigator.gpu.requestAdapter();
    // 请求GPU设备
    const device = await adapter?.requestDevice();
    if (!device) {
      throw new Error("Couldn't request WebGPU device");
    }

    // 创建画布元素
    const canvas = document.createElement("canvas");
    document.querySelector("#app")?.appendChild(canvas);

    // 获取WebGPU上下文
    const ctx = canvas.getContext("webgpu");
    if (!ctx) {
      throw new Error("Couldn't get WebGPU context");
    }

    // 获取首选画布格式
    const preferredFormat = navigator.gpu.getPreferredCanvasFormat();

    // 配置画布上下文
    ctx.configure({
      device,
      format: preferredFormat,
    });

    // 创建着色器模块
    const shader = device.createShaderModule({
      code: uniformWgsl, // 加载 WGSL 着色器代码
    });

    // 创建渲染管线
    const pipeline = device.createRenderPipeline({
      layout: "auto",
      vertex: {
        module: shader,
        entryPoint: "vs", // 顶点着色器入口
      },
      fragment: {
        module: shader,
        entryPoint: "fs", // 片元着色器入口
        targets: [
          {
            format: preferredFormat, // 渲染目标格式
          },
        ],
      },
    });

    // 使用工具函数解析着色器中的 uniform 数据定义
    const defs = makeShaderDataDefinitions(uniformWgsl);
    const params = makeStructuredView(defs.uniforms.params);

    // 设置 uniform 数据的初始值
    params.set({
      color: [1.0, 1.0, 0, 1], // 设置颜色为黄色
      scale: 0.5, // 缩放因子
      offset: [0.5, 0.5], // 偏移量
    });

    // 创建 GPU 缓冲区以存储 uniform 数据
    const buffer = device.createBuffer({
      size: params.arrayBuffer.byteLength, // 缓冲区大小与 uniform 数据大小一致
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST, // 用作 uniform 缓冲区并支持写入
    });

    // 将 uniform 数据写入缓冲区
    device.queue.writeBuffer(buffer, 0, params.arrayBuffer);

    // 创建绑定组，将 uniform 缓冲区绑定到着色器
    const bindGroup = device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0), // 获取绑定组布局
      entries: [
        {
          binding: 0, // 对应着色器中的 binding 位置
          resource: {
            buffer, // 绑定 uniform 缓冲区
          },
        },
      ],
    });

    return new WebGPUApp(
      device,
      device.queue,
      canvas,
      ctx,
      pipeline,
      bindGroup
    );
  }

  public render() {
    const { device, ctx, pipeline, bindGroup } = this;
    // 创建命令编码器（用于记录一系列GPU执行命令）
    const encoder = device.createCommandEncoder();

    // 获取当前Canvas的输出纹理（WebGPU渲染目标）
    const output = ctx.getCurrentTexture();
    const view = output.createView(); // 创建纹理视图用于渲染目标绑定

    // 开始渲染通道配置
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        // 配置颜色附件数组（此处仅使用一个主颜色目标）
        {
          view, // 绑定之前创建的纹理视图作为渲染目标
          clearValue: { r: 0, g: 0, b: 0, a: 1 }, // 设置清除颜色为黑色（RGB 0,0,0）
          loadOp: "clear", // 渲染前清除颜色缓冲区
          storeOp: "store", // 渲染完成后将结果存储到颜色缓冲区
        },
      ],
    });

    // 绑定当前渲染管线配置（顶点/片元着色器等）
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup); // 将 uniform 数据绑定到渲染管线
    // 执行绘制命令：绘制3个顶点构成的三角形
    // 参数3表示顶点数量（与顶点着色器中数组长度一致）
    pass.draw(3);

    // 结束当前渲染通道的配置
    pass.end();

    // 生成最终的命令缓冲区（包含所有已记录的渲染指令）
    const commandBuffer = encoder.finish(); // 修正拼写错误：commanderBuffer → commandBuffer
    device.queue.submit([commandBuffer]); // 将命令提交到GPU队列执行
  }
}

async function main() {
  const app = await WebGPUApp.create();

  // 使用 requestAnimationFrame 实现持续渲染
  const renderLoop = () => {
    app.render();
    requestAnimationFrame(renderLoop);
  };

  requestAnimationFrame(renderLoop);
}

// 调用主函数
main();
