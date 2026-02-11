use crate::{config::storage::ConfigStorage, esp::UpdateConfig, mqtt::init_mqtt_conn};
use prost::Message;
use rumqttc::{Event, EventLoop, Packet, QoS};

mod storage;

pub async fn run_config_loop() {
    let (client, mut eventloop) = init_mqtt_conn("config-worker");

    client
        .subscribe("showtime/config", QoS::AtMostOnce)
        .await
        .unwrap();

    let mut storage = ConfigStorage::load_or_new();

    loop {
        if let Some(config) = parse_msg(&mut eventloop).await {
            if config.is_ack {
                if let Some(present_config) = storage.get_device_config(&config.mac_addr) {
                    if !compare_configs(&present_config, &config) {
                        let payload = present_config.encode_to_vec();
                        let _ = client
                            .publish("showtime/config", QoS::AtMostOnce, true, payload)
                            .await;
                    }
                }
            } else {
                storage.set_device_config(config);
            }
        }
    }
}

fn compare_configs(a: &UpdateConfig, b: &UpdateConfig) -> bool {
    return a.device_name == b.device_name
        && a.dmx_universe == b.dmx_universe
        && a.dmx_lower_addr == b.dmx_lower_addr
        && a.dmx_upper_addr == b.dmx_upper_addr;
}

async fn parse_msg(eventloop: &mut EventLoop) -> Option<UpdateConfig> {
    if let Ok(notification) = eventloop.poll().await {
        if let Event::Incoming(Packet::Publish(p)) = notification {
            if let Ok(msg) = UpdateConfig::decode(&*p.payload) {
                return Some(msg);
            }
        }
    }
    return None;
}
