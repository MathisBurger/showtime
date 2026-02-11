use std::sync::mpsc::Receiver;

use eframe::egui;
use rumqttc::AsyncClient;

use crate::comm::{dto::EspDevice, run_mqtt_client};

pub fn render_setup<F>(ctx: &egui::Context, host: &mut String, port: &mut String, finish: F)
where
    F: FnOnce(Receiver<Vec<EspDevice>>, AsyncClient),
{
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Setup MQTT Connection");
            ui.add_space(20.0);

            ui.group(|ui| {
                ui.label("Broker Host:");
                ui.text_edit_singleline(host);

                ui.label("Port:");
                ui.text_edit_singleline(port);

                ui.add_space(10.0);

                if ui.button("🚀 Connect to Show").clicked() {
                    let (tx, rx) = std::sync::mpsc::channel();

                    let host_clone = host.clone();
                    let port_val = port.parse::<u16>().unwrap_or(1883);

                    let (client_tx, client_rx) = std::sync::mpsc::channel();

                    tokio::spawn(async move {
                        let client = run_mqtt_client(tx, host_clone, port_val).await;
                        let _ = client_tx.send(client);
                    });

                    if let Ok(client) = client_rx.recv() {
                        finish(rx, client);
                    }
                }
            });
        });
    });
}
