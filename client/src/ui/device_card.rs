use eframe::egui;

use crate::comm::dto::{DeviceStatus, EspDevice};

impl EspDevice {
    pub fn draw_device_card(&self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .fill(egui::Color32::from_gray(35))
            .rounding(10.0)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(60)))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let (status_text, status_color) = match self.status {
                            DeviceStatus::Online => ("● ONLINE", egui::Color32::GREEN),
                            DeviceStatus::Offline => ("○ OFFLINE", egui::Color32::RED),
                            DeviceStatus::Overdue => ("⚠ OVERDUE", egui::Color32::GOLD),
                        };

                        ui.heading(&self.device_name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(status_text)
                                    .color(status_color)
                                    .strong(),
                            );
                        });
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.monospace(&self.ip_addr);
                        ui.separator();
                        ui.label(format!("Jitter (sACN): {}ms", self.last_sacn_pkt));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (r, g, b) = self.color;
                            let display_color = egui::Color32::from_rgb(r as u8, g as u8, b as u8);
                            let size = egui::vec2(32.0, 32.0);
                            let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

                            ui.painter().circle_filled(
                                rect.center(),
                                rect.width() / 2.0,
                                display_color,
                            );
                            ui.add_space(8.0);
                        });
                    });

                    ui.add_space(8.0);

                    ui.add_space(4.0);

                    if !self.history.is_empty() {
                        ui.collapsing("History Log", |ui| {
                            for entry in self.history.iter().rev().take(5) {
                                ui.small(format!(
                                    "RGB({},{},{}) - Signal: {}%",
                                    entry.r, entry.g, entry.b, entry.signal_strength
                                ));
                            }
                        });
                    }
                });
            });
    }
}
