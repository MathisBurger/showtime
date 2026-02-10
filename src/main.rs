use std::sync::mpsc;

use crate::{comm::run_mqtt_client, ui::ShowtimeApp};

pub mod esp {
    include!(concat!(env!("OUT_DIR"), "/esp_status.rs"));
}

mod comm;
mod ui;

fn main() -> eframe::Result<()> {
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(run_mqtt_client(tx));
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Showtime Desktop",
        native_options,
        Box::new(|cc| Box::new(ShowtimeApp::new(cc, rx))),
    )
}
