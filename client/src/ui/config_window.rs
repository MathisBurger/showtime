use eframe::egui;
use prost::Message;
use rumqttc::AsyncClient;
use rumqttc::QoS;

use crate::comm::dto::EspDevice;
use crate::esp::UpdateConfig;

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

    pub fn render<F>(&mut self, ctx: &egui::Context, mqtt_client: Option<AsyncClient>, go_back: F)
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
                        if let Some(_client) = mqtt_client {
                            let config_msg = UpdateConfig {
                                mac_addr: self.device.mac_addr.clone(),
                                device_name: self.name.clone(),
                                dmx_universe: self.dmx_universe.parse::<u32>().unwrap(),
                                dmx_lower_addr: self.dmx_lower_addr.parse::<u32>().unwrap(),
                                dmx_upper_addr: self.dmx_upper_addr.parse::<u32>().unwrap(),
                                is_ack: false,
                            };
                            let payload = config_msg.encode_to_vec();

                            tokio::spawn(async move {
                                match _client
                                    .publish("showtime/config", QoS::AtLeastOnce, true, payload)
                                    .await
                                {
                                    Ok(_) => println!("Config sent successfully"),
                                    Err(e) => println!("Failed to publish config: {e}"),
                                }
                            });
                        }
                        go_back();
                    }
                });
            });
        });
    }
}
