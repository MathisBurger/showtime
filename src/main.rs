use crate::ui::MainWrapper;

pub mod esp {
    include!(concat!(env!("OUT_DIR"), "/esp_status.rs"));
}

mod comm;
mod common;
mod ui;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Showtime Desktop by Mathis Burger",
        native_options,
        Box::new(|_cc| Ok(Box::new(MainWrapper::new()))),
    )
}
