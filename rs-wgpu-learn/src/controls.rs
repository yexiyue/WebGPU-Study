#[derive(Debug, Clone)]
pub struct Controls {
    pub mag_filter: wgpu::FilterMode,
    pub address_mode_u: wgpu::AddressMode,
    pub address_mode_v: wgpu::AddressMode,
}

impl Controls {
    pub fn new() -> Self {
        Self {
            mag_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, mut on_change: impl FnMut(Self)) {
        egui::Window::new("Controls").show(ctx, |ui| {
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
                            on_change(self.clone());
                        }
                    })
            });

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
                            on_change(self.clone());
                        }
                    })
            });

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
                            on_change(self.clone());
                        }
                    })
            });
        });
    }
}
