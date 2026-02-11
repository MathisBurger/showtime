use eframe::egui;

use crate::comm::dto::EspDevice;

pub struct ConfigWindow {
    pub device: EspDevice,
    pub name: String,
    pub dmx_universe: String,
    pub dmx_lower_addr: String,
    pub dmx_upper_addr: String,
}

impl ConfigWindow {
    pub fn new(device: EspDevice) -> Self {
        Self {
            name: device.device_name.clone(),
            dmx_universe: device.dmx_universe.to_string(),
            dmx_lower_addr: device.dmx_lower_addr.to_string(),
            dmx_upper_addr: device.dmx_upper_addr.to_string(),
            device,
        }
    }

    pub fn render<F>(&mut self, ctx: &egui::Context, go_back: F)
    where
        F: FnOnce(),
    {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Setup Device config");
                ui.add_space(20.0);

                ui.group(|ui| {
                    ui.label("Device name:");
                    ui.text_edit_singleline(&mut self.name);

                    ui.label("DMX universe:");
                    ui.text_edit_singleline(&mut self.dmx_universe);

                    ui.label("DMX lower address:");
                    ui.text_edit_singleline(&mut self.dmx_lower_addr);

                    ui.label("DMX upper address:");
                    ui.text_edit_singleline(&mut self.dmx_upper_addr);

                    ui.add_space(10.0);

                    if ui.button("Update").clicked() {
                        go_back();
                    }
                });
            });
        });
    }
}
