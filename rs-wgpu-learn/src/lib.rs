use anyhow::Result;
use egui_wgpu::ScreenDescriptor;
use std::sync::Arc;
use wgpu::{Color, include_wgsl};
use winit::window::Window;
pub mod controls;
pub mod egui_render;

fn gen_texture_data() -> Vec<u8> {
    let red = [255u8, 0, 0, 255]; // 红色
    let yellow = [255, 255, 0, 255]; // 黄色
    let blue = [0, 0, 255, 255]; // 蓝色

    // // 定义二维纹理数据结构
    // let rows = [
    //     [blue, red, red, red, red],         // 第一行
    //     [red, yellow, yellow, yellow, red], // 第二行
    //     [red, yellow, red, red, red],       // 第三行
    //     [red, yellow, yellow, red, red],    // 第四行
    //     [red, yellow, red, red, red],       // 第五行
    //     [red, yellow, red, red, red],       // 第六行
    //     [red, red, red, red, red],          // 第七行
    // ];

    // 定义二维纹理数据结构并翻转
    let rows = [
        [red, red, red, red, red],          // 第七行
        [red, yellow, red, red, red],       // 第六行
        [red, yellow, red, red, red],       // 第五行
        [red, yellow, yellow, red, red],    // 第四行
        [red, yellow, red, red, red],       // 第三行
        [red, yellow, yellow, yellow, red], // 第二行
        [blue, red, red, red, red],         // 第一行
    ];

    // 将二维数组展平为一维字节数组
    rows.iter().flatten().flatten().copied().collect()
}

// Wgpu应用核心结构体
pub struct WgpuApp {
    pub window: Arc<Window>,                // 窗口对象
    pub surface: wgpu::Surface<'static>,    // GPU表面（用于绘制到窗口）
    pub device: wgpu::Device,               // GPU设备抽象
    pub queue: wgpu::Queue,                 // 命令队列（用于提交GPU命令）
    pub config: wgpu::SurfaceConfiguration, // 表面配置（格式、尺寸等）
    pub pipeline: wgpu::RenderPipeline,     // 渲染管线（包含着色器、状态配置等）
    pub bind_group: wgpu::BindGroup,
    pub texture: wgpu::Texture,
    pub egui_renderer: egui_render::EguiRender, // Egui渲染器
    pub controls: controls::Controls,
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

        let egui_render = egui_render::EguiRender::new(&device, config.format, None, 1, &window);

        // 6. 创建着色器模块（加载WGSL着色器）
        let shader = device.create_shader_module(include_wgsl!("../../source/texture.wgsl"));

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

        let texture_data = gen_texture_data();
        let texture_size = wgpu::Extent3d {
            width: 5,
            height: 7,
            ..Default::default()
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: texture_size,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texture_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(texture_size.width * 4),
                rows_per_image: None,
            },
            texture_size,
        );
        let controls = controls::Controls::new();

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: controls.address_mode_u,
            address_mode_v: controls.address_mode_v,
            mag_filter: controls.mag_filter,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&Default::default()),
                    ),
                },
            ],
        });

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            pipeline,
            bind_group,
            egui_renderer: egui_render,
            texture,
            controls,
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

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.window.as_ref().scale_factor() as f32,
        };

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

            // 6. 设置绑定组
            pass.set_bind_group(0, &self.bind_group, &[]);

            // 7. 使用实例化绘制
            pass.draw(0..6, 0..1);
        }

        {
            self.egui_renderer.begin_frame(&self.window);

            self.controls
                .render(self.egui_renderer.context(), |controls| {
                    let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                        address_mode_u: controls.address_mode_u,
                        address_mode_v: controls.address_mode_v,
                        mag_filter: controls.mag_filter,
                        ..Default::default()
                    });

                    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: None,
                        layout: &self.pipeline.get_bind_group_layout(0),
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::Sampler(&sampler),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(
                                    &self.texture.create_view(&Default::default()),
                                ),
                            },
                        ],
                    });
                    self.bind_group = bind_group;
                });

            self.egui_renderer.end_frame_and_draw(
                &self.device,
                &self.queue,
                &mut encoder,
                &self.window,
                &view,
                screen_descriptor,
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
