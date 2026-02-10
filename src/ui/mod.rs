use crate::esp::EspStatusMessage;
use eframe::Frame;
use eframe::egui::Context;
use std::sync::mpsc::Receiver;
use std::time::Duration;

pub struct ShowtimeApp {
    nodes: Vec<EspStatusMessage>,
    rx: Receiver<EspStatusMessage>,
}

impl ShowtimeApp {
    pub fn new(cc: &eframe::CreationContext<'_>, rx: Receiver<EspStatusMessage>) -> Self {
        Self {
            nodes: Vec::new(),
            rx,
        }
    }
}

impl eframe::App for ShowtimeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        while let Ok(msg) = self.rx.try_recv() {
            if let Some(node) = self
                .nodes
                .iter_mut()
                .find(|n| n.device_name == msg.device_name)
            {
                *node = msg;
            } else {
                self.nodes.push(msg);
            }
        }

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎬 Showtime: DMX Node Monitor");
            ui.add_space(10.0);

            for node in &self.nodes {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let color = eframe::egui::Color32::from_rgb(
                            node.r as u8,
                            node.g as u8,
                            node.b as u8,
                        );
                        let (rect, _) = ui.allocate_exact_size(
                            eframe::egui::vec2(20.0, 20.0),
                            eframe::egui::Sense::hover(),
                        );
                        ui.painter().circle_filled(rect.center(), 10.0, color);

                        ui.vertical(|ui| {
                            ui.label(format!("Gerät: {}", node.device_name));
                            ui.small(format!(
                                "sCAN Pkt: {} | Signal: {}%",
                                node.last_sacn_pkt, node.signal_strength
                            ));
                        });
                    });
                });
                ui.add_space(5.0);
            }
        });
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
