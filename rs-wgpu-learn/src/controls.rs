// 定义一个结构体 Controls，用于存储控件的状态
#[derive(Debug, Clone)]
pub struct Controls {
    pub mag_filter: wgpu::FilterMode,      // 纹理放大过滤模式
    pub address_mode_u: wgpu::AddressMode, // 纹理 U 轴寻址模式
    pub address_mode_v: wgpu::AddressMode, // 纹理 V 轴寻址模式
    pub image_url: String,                 // 图像 URL
}

impl Controls {
    // 创建一个新的 Controls 实例，初始化默认值
    pub fn new() -> Self {
        Self {
            mag_filter: wgpu::FilterMode::Nearest, // 默认使用最近点采样
            address_mode_u: wgpu::AddressMode::ClampToEdge, // 默认 U 轴边缘拉伸
            address_mode_v: wgpu::AddressMode::ClampToEdge, // 默认 V 轴边缘拉伸
            image_url: String::new(),              // 默认空字符串
        }
    }

    // 渲染控件的 UI
    pub fn render(
        &mut self,
        ctx: &egui::Context,             // egui 上下文
        mut on_change: impl FnMut(Self), // 当控件值改变时的回调函数
    ) {
        egui::Window::new("Controls").show(ctx, |ui| {
            // 渲染 Mag Filter 下拉框
            ui.horizontal(|ui| {
                ui.label("Mag Filter");
                egui::ComboBox::from_id_salt("mag_filter")
                    .selected_text(format!("{:?}", self.mag_filter))
                    .show_ui(ui, |ui| {
                        let a = ui
                            .selectable_value(
                                &mut self.mag_filter,
                                wgpu::FilterMode::Nearest,
                                "Nearest",
                            )
                            .changed();
                        let b = ui
                            .selectable_value(
                                &mut self.mag_filter,
                                wgpu::FilterMode::Linear,
                                "Linear",
                            )
                            .changed();
                        a || b
                    })
                    .inner
                    .map(|changed| {
                        if changed {
                            on_change(self.clone()); // 如果值改变，调用回调函数
                        }
                    })
            });
            ui.add_space(16.0); // 添加间距

            // 渲染 Address Mode U 下拉框
            ui.horizontal(|ui| {
                ui.label("Address Mode U");
                egui::ComboBox::from_id_salt("address_mode_u")
                    .selected_text(format!("{:?}", self.address_mode_u))
                    .show_ui(ui, |ui| {
                        let a = ui
                            .selectable_value(
                                &mut self.address_mode_u,
                                wgpu::AddressMode::ClampToEdge,
                                "ClampToEdge",
                            )
                            .changed();
                        let b = ui
                            .selectable_value(
                                &mut self.address_mode_u,
                                wgpu::AddressMode::Repeat,
                                "Repeat",
                            )
                            .changed();
                        a || b
                    })
                    .inner
                    .map(|changed| {
                        if changed {
                            on_change(self.clone()); // 如果值改变，调用回调函数
                        }
                    })
            });
            ui.add_space(16.0); // 添加间距

            // 渲染 Address Mode V 下拉框
            ui.horizontal(|ui| {
                ui.label("Address Mode V");

                egui::ComboBox::from_id_salt("address_mode_v")
                    .selected_text(format!("{:?}", self.address_mode_v))
                    .show_ui(ui, |ui| {
                        let a = ui
                            .selectable_value(
                                &mut self.address_mode_v,
                                wgpu::AddressMode::ClampToEdge,
                                "ClampToEdge",
                            )
                            .changed();
                        let b = ui
                            .selectable_value(
                                &mut self.address_mode_v,
                                wgpu::AddressMode::Repeat,
                                "Repeat",
                            )
                            .changed();
                        a || b
                    })
                    .inner
                    .map(|changed| {
                        if changed {
                            on_change(self.clone()); // 如果值改变，调用回调函数
                        }
                    })
            });
            ui.add_space(16.0); // 添加间距

            // 渲染 Image URL 文本框和加载按钮
            ui.vertical(|ui| {
                ui.label("Image URL");
                ui.add_space(8.0); // 添加间距
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.image_url); // 文本框输入 URL
                    if ui.button("Load").clicked() {
                        on_change(self.clone()); // 点击加载按钮时调用回调函数
                    }
                });
            })
        });
    }
}
