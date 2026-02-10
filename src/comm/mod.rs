pub mod dto;

use std::{collections::HashMap, sync::mpsc::Sender, time::Duration};

use prost::Message;
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};

use crate::{comm::dto::EspDevice, esp::EspStatusMessage};

pub async fn run_mqtt_client(tx: Sender<Vec<EspDevice>>) {
    let mut mqttoptions = MqttOptions::new("showtime_desktop", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client
        .subscribe("showtime/status", QoS::AtMostOnce)
        .await
        .unwrap();

    let mut device_map: HashMap<String, EspDevice> = HashMap::new();

    let mut ui_refresh_ticker = tokio::time::interval(Duration::from_millis(100));
    ui_refresh_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            notification = eventloop.poll() => {
                if let Ok(Event::Incoming(Packet::Publish(p))) = notification {
                    if let Ok(msg) = EspStatusMessage::decode(&*p.payload) {
                        let device = get_or_create_device(&msg, &mut device_map);
                        device.update_with_message(&msg);
                    }
                }
            },
            _ = ui_refresh_ticker.tick() => {
                for device in device_map.values_mut() {
                    device.update_status();
                }
                let values: Vec<EspDevice> = device_map.values().cloned().collect();
                if let Err(e) = tx.send(values) {
                    eprintln!("UI Thread wurde geschlossen, beende MQTT-Loop. {}", e);
                    break;
                }
            }
        }
    }
}

fn get_or_create_device<'a>(
    msg: &EspStatusMessage,
    devices: &'a mut HashMap<String, EspDevice>,
) -> &'a mut EspDevice {
    devices
        .entry(msg.ip_addr.clone())
        .or_insert_with(|| EspDevice::new(msg))
}
