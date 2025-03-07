use log::info;
use wgpu::{BufferUsages, InstanceDescriptor};

fn main() -> anyhow::Result<()> {
    // 初始化日志系统（仅显示INFO及以上级别日志）
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    // 使用pollster运行异步代码块（类似tokio的block_on）
    pollster::block_on(run())?;
    Ok(())
}

async fn run() -> anyhow::Result<()> {
    // 创建WGPU实例
    let instance = wgpu::Instance::new(&InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    // 请求适配器
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            ..Default::default()
        })
        .await
        .ok_or(anyhow::anyhow!("No suitable adapter found!"))?;

    // 请求设备和队列
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                memory_hints: wgpu::MemoryHints::Performance, // 内存优化策略
                ..Default::default()
            },
            None, // 不指定追踪路径
        )
        .await?;

    // 输入数据
    let input: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];

    // 创建存储缓冲区
    let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("storage_buffer"),
        size: input.len() as u64 * std::mem::size_of::<f32>() as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // 将输入数据写入存储缓冲区
    queue.write_buffer(&storage_buffer, 0, bytemuck::cast_slice(&input));

    // 创建结果缓冲区
    let result_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("result_buffer"),
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        size: storage_buffer.size(),
        mapped_at_creation: false,
    });

    // 创建着色器模块
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("computer_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../../source/compute.wgsl").into()),
    });

    // 创建绑定组布局
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("bind_group_layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(0),
            },
            count: None,
        }],
    });

    // 创建绑定组
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bind_group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: storage_buffer.as_entire_binding(),
        }],
    });

    // 创建计算管道布局
    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("compute_pipeline_layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    // 创建计算管道
    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("compute_pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        cache: None,
        compilation_options: Default::default(),
    });

    // 创建命令编码器
    let mut encode = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("encoder"),
    });
    {
        // 开始计算传递
        let mut pass = encode.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("compute_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&compute_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(input.len() as u32, 1, 1);
    }

    // 将存储缓冲区的数据复制到结果缓冲区
    encode.copy_buffer_to_buffer(&storage_buffer, 0, &result_buffer, 0, storage_buffer.size());

    // 提交命令缓冲区
    let command_buffer = encode.finish();
    queue.submit(std::iter::once(command_buffer));

    // 映射结果缓冲区并读取数据
    result_buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, |res| {
            info!("Mapping buffer ref {:?}", res);
        });
    device.poll(wgpu::Maintain::Wait);
    let res = result_buffer.slice(..).get_mapped_range().to_vec();
    let result: &[f32] = bytemuck::cast_slice(&res);
    info!("Result: {:?}", result);
    Ok(())
}