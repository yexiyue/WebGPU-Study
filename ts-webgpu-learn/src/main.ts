import "./style.css";
import storageWgsl from "../../source/vertex.wgsl?raw";

class WebGPUApp {
  constructor(
    public device: GPUDevice,
    public queue: GPUQueue,
    public canvas: HTMLCanvasElement,
    public ctx: GPUCanvasContext,
    public pipeline: GPURenderPipeline,
    public vertex_buffer: GPUBuffer,
    public instance_buffer: GPUBuffer,
    public index_buffer: GPUBuffer
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
      code: storageWgsl, // 加载 WGSL 着色器代码
    });

    // 创建渲染管线
    const pipeline = device.createRenderPipeline({
      layout: "auto",
      vertex: {
        module: shader,
        entryPoint: "vs", // 顶点着色器入口
        buffers: [Vertex.LAYOUT, Params.LAYOUT],
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
    const vertexData = new Float32Array(
      Vertex.SQUARE.flatMap((v) => v.position)
    );

    // 创建 GPU 缓冲区以存储 uniform 数据
    const vertexBuffer = device.createBuffer({
      size: vertexData.byteLength, // 缓冲区大小与 uniform 数据大小一致
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST, // 用作 uniform 缓冲区并支持写入
    });

    // 将 uniform 数据写入缓冲区
    device.queue.writeBuffer(vertexBuffer, 0, vertexData);

    const indexBuffer = device.createBuffer({
      size: Vertex.SQUARE_INDICES.byteLength,
      usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST,
    });

    device.queue.writeBuffer(indexBuffer, 0, Vertex.SQUARE_INDICES);

    const instanceData = new Float32Array(
      Array.from({ length: 10 })
        .map(() => Params.random())
        .flatMap((p) => [...p.color, ...p.offset, p.scale])
    );

    const instanceBuffer = device.createBuffer({
      size: instanceData.byteLength,
      usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST,
    });
    device.queue.writeBuffer(instanceBuffer, 0, instanceData);

    return new WebGPUApp(
      device,
      device.queue,
      canvas,
      ctx,
      pipeline,
      vertexBuffer,
      instanceBuffer,
      indexBuffer
    );
  }

  public render() {
    const { device, ctx, pipeline } = this;
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

    pass.setVertexBuffer(0, this.vertex_buffer); // 绑定顶点缓冲区
    pass.setVertexBuffer(1, this.instance_buffer); // 绑定实例缓冲区
    pass.setIndexBuffer(this.index_buffer, "uint16"); // 绑定索引缓冲区
    pass.drawIndexed(Vertex.SQUARE_INDICES.length, 10); // 绘制索引缓冲区中的图形

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

class Vertex {
  constructor(public position: [number, number]) {}

  static LAYOUT: GPUVertexBufferLayout = {
    arrayStride: 2 * 4,
    stepMode: "vertex",
    attributes: [
      {
        shaderLocation: 0,
        offset: 0,
        format: "float32x2",
      },
    ],
  };

  static SQUARE: Vertex[] = [
    new Vertex([-0.5, -0.5]),
    new Vertex([0.5, -0.5]),
    new Vertex([-0.5, 0.5]),
    new Vertex([0.5, 0.5]),
  ];

  static SQUARE_INDICES: Uint16Array = new Uint16Array([
    0,
    1,
    2, // Triangle 1
    2,
    1,
    3, // Triangle 2
  ]);
}

class Params {
  constructor(
    public color: [number, number, number, number],
    public offset: [number, number],
    public scale: number
  ) {}

  static LAYOUT: GPUVertexBufferLayout = {
    arrayStride: 4 * 7,
    stepMode: "instance",
    attributes: [
      {
        shaderLocation: 1,
        offset: 0,
        format: "float32x4",
      },
      {
        shaderLocation: 2,
        offset: 4 * 4,
        format: "float32x2",
      },
      {
        shaderLocation: 3,
        offset: 6 * 4,
        format: "float32",
      },
    ],
  };

  static random() {
    return new Params(
      [random(0, 1), random(0, 1), random(0, 1), 1],
      [random(-1, 1), random(-1, 1)],
      0.5
    );
  }
}

function random(start: number, end: number) {
  return Math.random() * (end - start) + start;
}
