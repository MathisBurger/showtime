use crate::{config::storage::ConfigStorage, esp::UpdateConfig, mqtt::init_mqtt_conn};
use prost::Message;
use rumqttc::{Event, EventLoop, Packet, QoS};

mod storage;

pub async fn run_config_loop() {
    let (client, mut eventloop) = init_mqtt_conn("config-worker");

    client
        .subscribe("showtime/config", QoS::AtLeastOnce)
        .await
        .unwrap();

    log::info!("Subscribing to config topic");

    let mut storage = ConfigStorage::load_or_new();

    log::info!("Setting config storage");

    loop {
        if let Some(config) = parse_msg(&mut eventloop).await {
            if config.is_ack {
                log::info!("Received config change ack for MAC: {}", config.mac_addr);
                if let Some(present_config) = storage.get_device_config(&config.mac_addr) {
                    if !compare_configs(&present_config, &config) {
                        log::info!("Sending config update for MAC: {}", config.mac_addr);

                        let payload = present_config.encode_to_vec();
                        let _ = client
                            .publish("showtime/config", QoS::AtMostOnce, true, payload)
                            .await;
                    }
                }
            } else {
                log::info!("Received config change for MAC: {}", config.mac_addr);
                storage.set_device_config(config);
            }
        }
    }
}

fn compare_configs(a: &UpdateConfig, b: &UpdateConfig) -> bool {
    return a.device_name == b.device_name && a.dmx_config == b.dmx_config;
}

async fn parse_msg(eventloop: &mut EventLoop) -> Option<UpdateConfig> {
    if let Ok(notification) = eventloop.poll().await {
        log::debug!("new msg");
        if let Event::Incoming(Packet::Publish(p)) = notification {
            log::debug!("Message extracted");
            if let Ok(msg) = UpdateConfig::decode(&*p.payload) {
                return Some(msg);
            }
        }
    }
    return None;
}
