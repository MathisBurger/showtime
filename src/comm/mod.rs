use std::{sync::mpsc::Sender, time::Duration};

use prost::Message;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};

use crate::esp::EspStatusMessage;

pub async fn run_mqtt_client(tx: Sender<EspStatusMessage>) {
    let mut mqttoptions = MqttOptions::new("showtime_desktop", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client
        .subscribe("showtime/status", QoS::AtMostOnce)
        .await
        .unwrap();

    loop {
        if let Ok(notification) = eventloop.poll().await {
            if let Event::Incoming(Packet::Publish(p)) = notification {
                if let Ok(decoded) = EspStatusMessage::decode(&*p.payload) {
                    let _ = tx.send(decoded);
                }
            }
        }
    }
}
