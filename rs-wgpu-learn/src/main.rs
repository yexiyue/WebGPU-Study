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
