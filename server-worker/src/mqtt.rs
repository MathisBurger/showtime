use std::time::Duration;

use rumqttc::{AsyncClient, EventLoop, MqttOptions};

pub fn init_mqtt_conn(name: &str) -> (AsyncClient, EventLoop) {
    let host = std::env::var("MQTT_HOST").unwrap();
    let port = std::env::var("MQTT_PORT").unwrap().parse::<u16>().unwrap();

    let mut mqttoptions = MqttOptions::new(name, host, port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    log::info!("Initializing new MQTT session with name: {}", name);

    AsyncClient::new(mqttoptions, 10)
}
