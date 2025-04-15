use anyhow::Result;
use std::sync::Arc;
use wgpu::{Color, include_wgsl, util::DeviceExt};
use winit::window::Window;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Params {
    /// size: 16, offset: 0, type: `vec4<f32>`
    pub color: [f32; 4],
    /// size: 8, offset: 16 (2*8), type: `vec2<f32>`
    pub offset: [f32; 2],
    /// size: 4, offset: 24 (4*6), type: `f32`
    pub scale: f32,
    // pub _pad_scale: [u8; 0x8 - core::mem::size_of::<f32>()],
}

impl Params {
    pub fn new(color: [f32; 4], offset: [f32; 2], scale: f32) -> Self {
        Self {
            color,
            offset,
            scale,
            // _pad_scale: [0; 0x8 - core::mem::size_of::<f32>()],
        }
    }

    pub fn random() -> Self {
        // 随机生成颜色
        let color = [
            rand::random_range(0.0..=1.0),
            rand::random_range(0.0..=1.0),
            rand::random_range(0.0..=1.0),
            1.0,
        ];
        // 随机生成偏移量
        let offset = [
            rand::random_range(-1.0..=1.0),
            rand::random_range(-1.0..=1.0),
        ];
        // 随机生成缩放因子
        let scale = 0.5;

        Self {
            color,
            offset,
            scale,
            // _pad_scale: [0; 0x8 - core::mem::size_of::<f32>()],
        }
    }

    /// 定义Params结构体的顶点缓冲区布局，用于多实例渲染参数传递
    ///
    /// # 配置说明：
    /// - `array_stride`：结构体总字节大小（16字节）
    /// - `step_mode`：每个实例使用新数据（Instance模式）
    /// - `attributes`：与WGSL中@location标记的字段一一对应
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        // 结构体总字节长度（4*4 + 2*4 + 1*4 = 16字节）
        array_stride: std::mem::size_of::<Params>() as wgpu::BufferAddress,

        // 实例步进模式：每个实例获取新数据
        step_mode: wgpu::VertexStepMode::Instance,

        // 属性映射配置：
        // 使用vertex_attr_array宏简化定义
        // 格式与WGSL中@location标记的字段对应：
        //   1 → color（vec4f）
        //   2 → offset（vec2f）
        //   3 → scale（f32）
        attributes: &wgpu::vertex_attr_array![
            1 => Float32x4,  // 对应@location(1) color
            2 => Float32x2,  // 对应@location(2) offset
            3 => Float32     // 对应@location(3) scale
        ],
    };
}

// Wgpu应用核心结构体
pub struct WgpuApp {
    pub window: Arc<Window>,                // 窗口对象
    pub surface: wgpu::Surface<'static>,    // GPU表面（用于绘制到窗口）
    pub device: wgpu::Device,               // GPU设备抽象
    pub queue: wgpu::Queue,                 // 命令队列（用于提交GPU命令）
    pub config: wgpu::SurfaceConfiguration, // 表面配置（格式、尺寸等）
    pub pipeline: wgpu::RenderPipeline,     // 渲染管线（包含着色器、状态配置等）
    pub instance_length: u32,
    pub vertex_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
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
            .await?;

        // 4. 创建设备和命令队列
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
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
        let shader = device.create_shader_module(include_wgsl!("../../source/vertex.wgsl"));

        // 7. 创建渲染管线

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: None, // 使用默认管线布局
            vertex: wgpu::VertexState {
                module: &shader,                            // 顶点着色器模块
                entry_point: Some("vs"),                    // 入口函数
                buffers: &[Vertex::LAYOUT, Params::LAYOUT], // 顶点缓冲区布局
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

        let instance_length = 10;
        let params_list = (0..instance_length)
            .map(|_| Params::random())
            .collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&params_list),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: None,
        //     layout: &pipeline.get_bind_group_layout(0),
        //     entries: &[wgpu::BindGroupEntry {
        //         binding: 0,
        //         resource: buffer.as_entire_binding(),
        //     }],
        // });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&Vertex::SQUARE),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&Vertex::SQUARE_INDEXED),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            pipeline,
            instance_length,
            vertex_buffer,
            instance_buffer,
            index_buffer,
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

            pass.set_pipeline(&self.pipeline);
            // pass.set_bind_group(0, &self.bind_group, &[]);

            // 设置顶点缓冲区
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // vertices参数要与TRIANGLE的长度一致
            // pass.draw(0..Vertex::SQUARE.len() as u32, 0..self.instance_length);
            pass.draw_indexed(
                0..Vertex::SQUARE_INDEXED.len() as u32,
                0,
                0..self.instance_length,
            );
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

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

#[allow(dead_code)]
impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            // 顶点数据步长（每个顶点占字节数）
            // 计算Vertex结构体的大小（2x4字节 = 8字节）
            // 告诉GPU每个顶点数据在缓冲区中占据的字节数，用于逐顶点读取
            array_stride: core::mem::size_of::<Vertex>() as wgpu::BufferAddress,

            // 步进模式：每个顶点使用新的数据
            // VertexStepMode::Vertex表示每个顶点都会获取新的属性值
            step_mode: wgpu::VertexStepMode::Vertex,

            // 顶点属性数组：定义顶点数据如何映射到着色器
            // 此处配置了一个属性：
            // - offset: 0（从缓冲区起始位置开始）
            // - shader_location: 0（对应着色器中location=0的属性）
            // - format: Float32x2（2个32位浮点数，对应position字段）
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }

    // 可以通过`wgpu::vertex_attr_array!`宏来简化描述
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: core::mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x2
        ],
    };

    pub const TRIANGLE: [Vertex; 3] = [
        Vertex {
            position: [0.0, 0.5],
        },
        Vertex {
            position: [-0.5, -0.5],
        },
        Vertex {
            position: [0.5, -0.5],
        },
    ];

    // // 0--1 4
    // // | / /|
    // // |/ / |
    // // 2 3--5
    // pub const SQUARE: [Vertex; 6] = [

    //     // 第一个三角形
    //     Vertex {
    //         position: [-0.5, -0.5],
    //     },
    //     Vertex {
    //         position: [0.5, -0.5],
    //     },
    //     Vertex {
    //         position: [-0.5, 0.5],
    //     },
    //     // 第二个三角形
    //     Vertex {
    //         position: [-0.5, 0.5],
    //     },
    //     Vertex {
    //         position: [0.5, -0.5],
    //     },
    //     Vertex {
    //         position: [0.5, 0.5],
    //     },
    // ];

    // 0--1
    // | /|
    // |/ |
    // 2--3
    pub const SQUARE: [Vertex; 4] = [
        // 第一个三角形
        Vertex {
            position: [-0.5, -0.5],
        },
        Vertex {
            position: [0.5, -0.5],
        },
        Vertex {
            position: [-0.5, 0.5],
        },
        Vertex {
            position: [0.5, 0.5],
        },
    ];

    pub const SQUARE_INDEXED: [u16; 6] = [0, 1, 2, 2, 1, 3];
}
