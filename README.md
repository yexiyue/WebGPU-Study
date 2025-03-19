## 三、WebGPU 基础入门——绘制三角型

本节将通过一个简单的例子，带你一步步认识WebGPU的渲染管线，并学习如何编写顶点着色器和片元着色器。我们将使用TypeScript（TS）和Rust（RS）两种编程语言来实现这个例子。

由于两种语言渲染的环境不一样，这节就分开来讲解。

### 1. TypeScript 实现

在开始之前，我们需要先修改一下style.css文件，将画布的尺寸设置为100%。

```css
canvas {
  border: 1px solid black;
  width: 100vw;
  height: 100vh;
}

body,
html {
  margin: 0;
  padding: 0;
  overflow: hidden;
}

```

我们接着上一节的代码，将main函数清空，然后请求GPU适配器和设备。

```ts
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
}

```

接着我们创建一个`Canvas`元素，并获取`WebGPU`的上下文。

```ts
// 创建画布元素
const canvas = document.createElement("canvas");
document.querySelector("#app")?.appendChild(canvas);

// 获取WebGPU上下文
const ctx = canvas.getContext("webgpu");
if (!ctx) {
  throw new Error("Couldn't get WebGPU context");
}
```

然后我们配置画布的纹理格式。**TextureFormat（格式）**是WebGPU中定义颜色缓冲区像素存储方式的核心参数，决定了每个像素的位深、颜色空间和内存布局。通过调用`getPreferredCanvasFormat()`，浏览器会根据当前设备的硬件特性（如GPU架构、驱动支持）自动返回最优的纹理格式，从而确保渲染性能和兼容性。

```ts
// 获取浏览器推荐的最优画布格式（自动适配设备最佳渲染格式）
const preferredFormat = navigator.gpu.getPreferredCanvasFormat();

// 配置画布上下文，绑定设备并指定纹理格式
ctx.configure({
  device,
  format: preferredFormat, // 格式决定了颜色精度、内存占用和渲染管线兼容性
});

```

接着在`source/triangle.wgsl`文件中编写顶点着色器和片元着色器。

顶点着色器`vs`:根据 vertexIndex 从 pos 数组中选择对应的顶点坐标，形成三角形的三个顶点。

```wgsl
@vertex
fn vs(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    var pos = array(vec2f(0.0, 0.5), // 顶点1（顶部中心）
    vec2f(-0.5, -0.5), // 顶点1（顶部中心）
    vec2f(0.5, -0.5) // 顶点1（顶部中心）
    );
    return vec4f(pos[vertex_index], 0.0, 1.0); // 扩展为4D向量
}
```

- vertexIndex：通过 @builtin(vertex_index) 获取当前顶点索引（0、1、2），对应三角形的三个顶点。
- 返回 vec4f 向量，前两个分量 (x, y) 是顶点的二维坐标，z 设为 0.0（假设在XY平面），w 设为 1.0（符合齐次坐标规范）。
- 通过 @builtin(position) 标记，此向量表示顶点在裁剪空间中的位置：
  X 范围：-1.0（左）到 +1.0（右）
  Y 范围：-1.0（底）到 +1.0（顶）

片段着色器`fs`:计算每个像素（片元）的最终颜色，决定渲染效果。

```wgsl
@fragment
fn fs() -> @location(0) vec4f {
    return vec4f(1.0, 0.0, 0.0, 1.0); // 纯红色（不透明）
}
```

- 返回 vec4f 表示颜色，格式为 RGBA：R=1.0（红色），G=0，B=0，A=1.0（不透明）。
- 通过 @location(0) 标记，结果写入渲染管线的第一个颜色目标（如画布）。

接着创建着色器模块

```ts
// 在文件顶部导入着色器文件
import triangle from "../../source/triangle.wgsl?raw";

//...
const shader = device.createShaderModule({
  code: triangle,
});
```

然后创建渲染管线
在WebGPU中，渲染管线（Render Pipeline）是一组预定义的配置，用于控制图形渲染的整个流程，从顶点数据输入到最终像素输出。

```ts
// 创建渲染管线
const pipeline = device.createRenderPipeline({
  layout: "auto",
  vertex: {
    module: shader,
    entryPoint: "vs",
  },
  fragment: {
    module: shader,
    entryPoint: "fs",
    targets: [
      {
        format: preferredFormat,
      },
    ],
  },
});
```

接下来创建渲染命令

```ts
// main函数中...
function render(){
  // 创建命令编码器（用于记录一系列GPU执行命令）
  const encoder = device.createCommandEncoder();

  // 获取当前Canvas的输出纹理（WebGPU渲染目标）
  const output = ctx.getCurrentTexture();
  const view = output.createView(); // 创建纹理视图用于渲染目标绑定

  // 开始渲染通道配置
  const pass = encoder.beginRenderPass({
    colorAttachments: [ // 配置颜色附件数组（此处仅使用一个主颜色目标）
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

  // 执行绘制命令：绘制3个顶点构成的三角形
  // 参数3表示顶点数量（与顶点着色器中数组长度一致）
  pass.draw(3);

  // 结束当前渲染通道的配置
  pass.end();

  // 生成最终的命令缓冲区（包含所有已记录的渲染指令）
  const commandBuffer = encoder.finish(); 
  device.queue.submit([commandBuffer]); // 将命令提交到GPU队列执行
}
```

最后运行该代码，可以看到一个红色的三角形。

如果你放大浏览器窗口，可能会发现三角形的边缘是块状的。这是因为`canvas`标签的默认分辨率为 300x150 像素。我们希望调整画布的分辨率，使其与显示的尺寸相匹配。

使用ResizeObserver来监听画布尺寸的变化，并根据新的尺寸重新设置画布的分辨率。

```ts
const observer = new ResizeObserver(entries => {
  for (const entry of entries) {
    const canvas = entry.target;
    const width = entry.contentBoxSize[0].inlineSize;
    const height = entry.contentBoxSize[0].blockSize;
    canvas.width = Math.max(1, Math.min(width, device.limits.maxTextureDimension2D));
    canvas.height = Math.max(1, Math.min(height, device.limits.maxTextureDimension2D));
    // 重新绘制
    render();
  }
});
observer.observe(canvas);
```

最后整理一下代码，封装成一个类`WebGPUApp`。最终代码如下：

```ts
import "./style.css";
import triangle from "../../source/triangle.wgsl?raw";

class WebGPUApp {
  constructor(
    public device: GPUDevice,
    public queue: GPUQueue,
    public canvas: HTMLCanvasElement,
    public ctx: GPUCanvasContext,
    public pipeline: GPURenderPipeline
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
      code: triangle,
    });

    // 创建渲染管线
    const pipeline = device.createRenderPipeline({
      layout: "auto",
      vertex: {
        module: shader,
        entryPoint: "vs",
      },
      fragment: {
        module: shader,
        entryPoint: "fs",
        targets: [
          {
            format: preferredFormat,
          },
        ],
      },
    });
    return new WebGPUApp(device, device.queue, canvas, ctx, pipeline);
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

```

由于获取`adapter`和`device`都是异步操作，但是不能在构造函数中使用`async`关键字，所以我们使用了`create`静态方法来创建`WebGPUApp`实例。在构造函数中，使用`ResizeObserver`来监听画布尺寸的变化，并根据新的尺寸重新设置画布的分辨率。

### 2. Rust 实现

开始之前先确定依赖是否安装好，Cargo.toml文件如下：

```toml
[package]
name = "rs-wgpu-learn"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1.0.97"
bytemuck = { version = "1.22.0", features = ["derive"] }
env_logger = "0.11.6"
log = "0.4.26"
parking_lot = "0.12.3"
pollster = "0.4.0"
wgpu = "24.0.1"
winit = "0.30.9"
```

然后在`lib.rs`中编写代码：
由于Rust中没有类的概念，所以我们使用结构体来封装WebGPU相关的数据。

```rust
use anyhow::Result;
use std::sync::Arc;
use wgpu::{Color, include_wgsl};
use winit::window::Window;

// Wgpu应用核心结构体
pub struct WgpuApp {
    pub window: Arc<Window>,                // 窗口对象
    pub surface: wgpu::Surface<'static>,    // GPU表面（用于绘制到窗口）
    pub device: wgpu::Device,               // GPU设备抽象
    pub queue: wgpu::Queue,                 // 命令队列（用于提交GPU命令）
    pub config: wgpu::SurfaceConfiguration, // 表面配置（格式、尺寸等）
    pub pipeline: wgpu::RenderPipeline,     // 渲染管线（包含着色器、状态配置等）
}

impl WgpuApp {
    /// 异步构造函数：初始化WebGPU环境
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        // 1. 创建WebGPU实例
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        // 2. 创建窗口表面
        let surface = instance.create_surface(window.clone())?;

        // 3. 请求图形适配器（选择GPU）
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(), // 默认选择高性能GPU
                compatible_surface: Some(&surface),                 // 需要与表面兼容
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("No adapter found"))?;

        // 4. 创建设备和命令队列
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await?;

        // 5. 配置表面（设置像素格式、尺寸等）
        let config = surface
            .get_default_config(
                &adapter,
                window.inner_size().width.max(1),  // 确保最小宽度为1
                window.inner_size().height.max(1), // 确保最小高度为1
            )
            .unwrap();
        surface.configure(&device, &config);

        // 6. 创建着色器模块（加载WGSL着色器）
        let shader = device.create_shader_module(include_wgsl!("../../source/triangle.wgsl"));

        // 7. 创建渲染管线
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: None, // 使用默认管线布局
            vertex: wgpu::VertexState {
                module: &shader,         // 顶点着色器模块
                entry_point: Some("vs"), // 入口函数
                buffers: &[],            // 顶点缓冲区布局（本示例为空）
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,         // 片元着色器模块
                entry_point: Some("fs"), // 入口函数
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,                  // 使用表面配置的格式
                    blend: Some(wgpu::BlendState::REPLACE), // 混合模式：直接替换
                    write_mask: wgpu::ColorWrites::ALL,     // 允许写入所有颜色通道
                })],
                compilation_options: Default::default(),
            }),
            primitive: Default::default(), // 使用默认图元配置（三角形列表）
            depth_stencil: None,           // 禁用深度/模板测试
            multisample: Default::default(), // 多重采样配置
            multiview: None,
            cache: None,
        });

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            pipeline,
        })
    }

    /// 执行渲染操作
    pub fn render(&mut self) -> Result<()> {
        // 1. 获取当前帧缓冲区
        let output = self.surface.get_current_texture()?;

        // 2. 创建纹理视图
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // 3. 创建命令编码器
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        // 4. 开始渲染通道
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(Color::BLACK), // 用黑色清除背景
                        store: wgpu::StoreOp::Store,             // 存储渲染结果
                    },
                    resolve_target: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // 5. 设置渲染管线
            pass.set_pipeline(&self.pipeline);

            // 6. 绘制调用（绘制3个顶点，组成一个三角形）
            pass.draw(0..3, 0..1);
        }

        // 7. 提交命令到队列
        let command_buffer = encoder.finish();
        self.queue.submit(std::iter::once(command_buffer));

        // 8. 呈现渲染结果
        output.present();

        Ok(())
    }

    /// 处理窗口大小变化
    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);
        // 重新配置表面（更新尺寸）
        self.surface.configure(&self.device, &self.config);
    }
}
```

- 初始化阶段：创建WebGPU实例→适配器→设备/队列→配置表面→创建渲染管线
- 渲染循环：获取帧缓冲区→创建命令→设置渲染通道→执行绘制→提交命令→呈现结果
- 窗口调整：更新表面配置确保渲染尺寸与窗口匹配

然后在`main.rs`中编写下面代码：

```rust
use log::info;
use parking_lot::Mutex;
use rs_wgpu_learn::WgpuApp;
use std::{rc::Rc, sync::Arc};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop,
    window::WindowAttributes,
};

fn main() -> anyhow::Result<()> {
    // 初始化日志系统（配置为仅显示INFO及以上级别的日志）
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    // 创建事件循环（窗口系统的核心事件处理器）
    let event_loop = EventLoop::new()?;
    // 创建应用实例并运行事件循环
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}

// 主应用结构体
#[derive(Default)]
struct App {
    /// WGPU应用实例的共享引用（使用 Rc + Mutex 实现跨线程安全访问）
    wgpu_app: Rc<Mutex<Option<WgpuApp>>>,
}

// ApplicationHandler trait 是 winit 窗口库的核心事件处理接口，主要用于管理应用程序生命周期和窗口事件。
impl ApplicationHandler for App {
    /// 当应用恢复/启动时触发（主要初始化入口）
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        info!("Resumed");
        // 防止重复初始化
        if self.wgpu_app.lock().is_some() {
            return;
        }

        // 1. 创建窗口
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default().with_title("Wgpu Learn"), // 设置窗口标题
                )
                .unwrap(),
        );

        // 2. 同步初始化WGPU应用（使用pollster阻塞等待异步初始化）
        let wgpu_app = pollster::block_on(WgpuApp::new(window)).unwrap();

        // 3. 存储WGPU应用实例
        self.wgpu_app.lock().replace(wgpu_app);
    }

    /// 处理窗口事件（核心事件循环）
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let mut app_guard = self.wgpu_app.lock();
        // 确保WGPU应用已初始化
        if app_guard.is_none() {
            return;
        }
        let app = app_guard.as_mut().unwrap();

        match event {
            // 关闭窗口请求
            WindowEvent::CloseRequested => {
                info!("Window close requested");
                event_loop.exit(); // 退出事件循环
            }

            // 重绘请求（驱动渲染循环）
            WindowEvent::RedrawRequested => {
                // 执行窗口预呈现通知
                app.window.pre_present_notify();

                // 执行实际渲染操作
                app.render().unwrap();

                // 请求下一帧重绘（维持持续渲染）
                app.window.request_redraw();
            }

            // 窗口大小变化事件
            WindowEvent::Resized(size) => {
                // 更新WGPU表面配置
                app.resize(size);
                info!("Window resized to {:?}", size);
            }

            // 其他未处理事件
            _ => {}
        }
    }
}
```

最后在`rs-wgpu-learn`文件夹下运行`cargo run`命令来启动应用程序。

### 最后

本节源码位于[Github](https://github.com/yexiyue/WebGPU-Study/tree/3-triangle)。
**如果本文对你有启发，欢迎点赞⭐收藏📚关注👀**，你的支持是我持续创作深度技术内容的最大动力。
