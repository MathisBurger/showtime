use crate::config::run_config_loop;

pub mod esp {
    include!(concat!(env!("OUT_DIR"), "/esp_status.rs"));
}

mod config;
mod mqtt;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    pretty_env_logger::init();

    run_config_loop().await;
}
