use crate::comm::dto::EspDevice;
use eframe::Frame;
use eframe::egui;
use eframe::egui::Context;
use std::sync::mpsc::Receiver;
use std::time::Duration;

pub struct ShowtimeApp {
    rx: Receiver<Vec<EspDevice>>,
    devices: Vec<EspDevice>,
}

impl ShowtimeApp {
    pub fn new(rx: Receiver<Vec<EspDevice>>) -> Self {
        Self {
            rx,
            devices: Vec::new(),
        }
    }
}

impl eframe::App for ShowtimeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        while let Ok(new_devices) = self.rx.try_recv() {
            self.devices = new_devices;
            self.devices.sort_by_key(|dev| dev.ip_addr.clone());
        }

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎬 Showtime: LED Status Monitor");
            ui.add_space(10.0);

            eframe::egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let available_width = ui.available_width();
                    let card_width = 320.0;
                    let spacing = 12.0;
                    let num_columns =
                        (available_width / (card_width + spacing)).max(1.0).floor() as usize;

                    egui::Grid::new("device_grid")
                        .num_columns(num_columns)
                        .spacing([spacing, spacing])
                        .show(ui, |ui| {
                            for (i, device) in self.devices.iter().enumerate() {
                                ui.vertical(|ui| {
                                    ui.set_width(card_width);
                                    ui.push_id(&device.ip_addr, |ui| {
                                        device.draw_device_card(ui);
                                    });
                                });
                                if (i + 1) % num_columns == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });
        });
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
