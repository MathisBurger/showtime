use eframe::egui;

use crate::comm::dto::EspDevice;

pub fn render_set_config<F>(ctx: &egui::Context, device: EspDevice, go_back: F)
where
    F: FnOnce(),
{
    let mut name = device.device_name.clone();
    let mut dmx_universe = device.dmx_universe.to_string();
    let mut dmx_lower_addr = device.dmx_lower_addr.to_string();
    let mut dmx_upper_addr = device.dmx_upper_addr.to_string();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Setup Device config");
            ui.add_space(20.0);

            ui.group(|ui| {
                ui.label("Device name:");
                ui.text_edit_singleline(&mut name);

                ui.label("DMX universe:");
                ui.text_edit_singleline(&mut dmx_universe);

                ui.label("DMX lower address:");
                ui.text_edit_singleline(&mut dmx_lower_addr);

                ui.label("DMX upper address:");
                ui.text_edit_singleline(&mut dmx_upper_addr);

                ui.add_space(10.0);

                if ui.button("Update").clicked() {
                    go_back();
                }
            });
        });
    });
}
