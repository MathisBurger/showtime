use eframe::egui;
use prost::Message;
use rumqttc::AsyncClient;
use rumqttc::QoS;

use crate::comm::dto::EspDevice;
use crate::esp::{DmxConfig, UpdateConfig};

#[derive(Clone)]
pub struct DmxConfigEdit {
    pub universe: String,
    pub start_addr: String,
    pub led_count: String,
    pub esp_pin: String,
    pub mode: String,
}

impl DmxConfigEdit {
    pub fn new() -> Self {
        Self {
            universe: "1".to_string(),
            start_addr: "1".to_string(),
            led_count: "50".to_string(),
            esp_pin: "2".to_string(),
            mode: "0".to_string(),
        }
    }

    pub fn from_proto(config: &DmxConfig) -> Self {
        Self {
            universe: config.universe.to_string(),
            start_addr: config.start_addr.to_string(),
            led_count: config.led_count.to_string(),
            esp_pin: config.esp_pin.to_string(),
            mode: config.mode.to_string(),
        }
    }

    pub fn to_proto(&self) -> Option<DmxConfig> {
        Some(DmxConfig {
            universe: self.universe.parse().ok()?,
            start_addr: self.start_addr.parse().ok()?,
            led_count: self.led_count.parse().ok()?,
            esp_pin: self.esp_pin.parse().ok()?,
            mode: self.mode.parse().ok()?,
        })
    }
}

pub struct ConfigWindow {
    pub device: EspDevice,
    pub name: String,
    pub dmx_configs: Vec<DmxConfigEdit>,
}

impl ConfigWindow {
    pub fn new(device: EspDevice) -> Self {
        let dmx_configs = if device.dmx_configs.is_empty() {
            vec![DmxConfigEdit::new()]
        } else {
            device
                .dmx_configs
                .iter()
                .map(DmxConfigEdit::from_proto)
                .collect()
        };

        Self {
            name: device.device_name.clone(),
            dmx_configs,
            device,
        }
    }

    pub fn render<F>(
        &mut self,
        ctx: &egui::Context,
        mqtt_client: Option<AsyncClient>,
        mut go_back: F,
    ) where
        F: FnMut(),
    {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Device Configuration");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("MAC Address:");
                    ui.label(&self.device.mac_addr);
                });

                ui.add_space(20.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.group(|ui| {
                        ui.label("Device Name:");
                        ui.text_edit_singleline(&mut self.name);
                    });

                    ui.add_space(15.0);

                    ui.heading("DMX Outputs");
                    ui.add_space(10.0);

                    let mut to_remove = None;

                    let num_configs = self.dmx_configs.len();
                    for (idx, config) in self.dmx_configs.iter_mut().enumerate() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.heading(format!("Output #{}", idx + 1));
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("🗑️ Remove").clicked() && num_configs > 1
                                        {
                                            to_remove = Some(idx);
                                        }
                                    },
                                );
                            });

                            ui.add_space(5.0);

                            egui::Grid::new(format!("dmx_config_{}", idx))
                                .num_columns(2)
                                .spacing([10.0, 8.0])
                                .show(ui, |ui| {
                                    ui.label("Universe:");
                                    ui.text_edit_singleline(&mut config.universe);
                                    ui.end_row();

                                    ui.label("Start Address:");
                                    ui.text_edit_singleline(&mut config.start_addr);
                                    ui.end_row();

                                    ui.label("LED Count:");
                                    ui.text_edit_singleline(&mut config.led_count);
                                    ui.end_row();

                                    ui.label("ESP Pin:");
                                    ui.text_edit_singleline(&mut config.esp_pin);
                                    ui.end_row();

                                    ui.label("Mode:");
                                    ui.text_edit_singleline(&mut config.mode);
                                    ui.end_row();
                                });
                        });

                        ui.add_space(10.0);
                    }

                    if let Some(idx) = to_remove {
                        self.dmx_configs.remove(idx);
                    }

                    if ui.button("➕ Add DMX Output").clicked() {
                        self.dmx_configs.push(DmxConfigEdit::new());
                    }

                    ui.add_space(20.0);

                    ui.horizontal(|ui| {
                        if ui.button("💾 Update Configuration").clicked() {
                            if let Some(client) = mqtt_client {
                                let dmx_configs: Vec<DmxConfig> = self
                                    .dmx_configs
                                    .iter()
                                    .filter_map(|c| c.to_proto())
                                    .collect();

                                let config_msg = UpdateConfig {
                                    mac_addr: self.device.mac_addr.clone(),
                                    device_name: self.name.clone(),
                                    dmx_config: dmx_configs,
                                    is_ack: false,
                                };

                                let payload = config_msg.encode_to_vec();

                                tokio::spawn(async move {
                                    match client
                                        .publish(
                                            "showtime/config",
                                            QoS::AtLeastOnce,
                                            false,
                                            payload,
                                        )
                                        .await
                                    {
                                        Ok(_) => println!("Config sent successfully"),
                                        Err(e) => eprintln!("Failed to publish config: {}", e),
                                    }
                                });
                            }
                            go_back();
                        }

                        if ui.button("❌ Cancel").clicked() {
                            go_back();
                        }
                    });
                });
            });
        });
    }
}
