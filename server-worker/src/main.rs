use crate::config::run_config_loop;

pub mod esp {
    include!(concat!(env!("OUT_DIR"), "/esp_status.rs"));
}

mod config;
mod mqtt;
mod dmx;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    // Start sACN/DMX Receiver in a separate thread
    tokio::spawn(async move {
        dmx::run_dmx_loop();
    });

    run_config_loop().await;
}
