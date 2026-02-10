use prost::Message;

use crate::esp::EspStatusMessage;

pub mod esp {
    include!(concat!(env!("OUT_DIR"), "/esp_status.rs"));
}

fn main() {
    println!("Hello, world!");
}
