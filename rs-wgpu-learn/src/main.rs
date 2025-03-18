use log::info;
use parking_lot::Mutex;
use rs_wgpu_learn::WgpuApp;
use std::{rc::Rc, sync::Arc};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop,
    window::WindowAttributes,
};

fn main() -> anyhow::Result<()> {
    // 初始化日志系统（仅显示INFO及以上级别日志）
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let event_loop = EventLoop::new()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}

#[derive(Default)]
struct App {
    wgpu_app: Rc<Mutex<Option<WgpuApp>>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        info!("Resumed");
        if self.wgpu_app.lock().is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("Wgpu Learn"))
                .unwrap(),
        );
        let wgpu_app = pollster::block_on(WgpuApp::new(window)).unwrap();
        self.wgpu_app.lock().replace(wgpu_app);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let mut app = self.wgpu_app.lock();
        if app.is_none() {
            return;
        }
        let app = app.as_mut().unwrap();

        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                app.window.pre_present_notify();
                // info!("Redraw requested");
                app.render().unwrap();

                app.window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                app.resize(size);

                info!("Window resized to {:?}", size);
            }
            _ => {}
        }
    }
}
