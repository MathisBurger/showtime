use prost::Message;
use rumqttc::QoS;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::{Duration, Instant};

use crate::config::storage::ConfigStorage;
use crate::esp::{DmxConfig, SetDmx, UpdateConfig, set_dmx::OutputData};
use crate::mqtt::init_mqtt_conn;

pub struct SacnReceiver {
    socket: UdpSocket,
    universe_cache: HashMap<u16, Vec<u8>>,
    storage: ConfigStorage,
    last_storage_load: Instant,
    mqtt_client: rumqttc::AsyncClient,
}

impl SacnReceiver {
    pub fn new(port: u16) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", port))?;
        socket.set_nonblocking(false)?;

        let (client, mut eventloop) = init_mqtt_conn("dmx-worker");

        // Background MQTT loop to handle pings and published messages
        tokio::spawn(async move {
            loop {
                if let Err(e) = eventloop.poll().await {
                    log::error!("MQTT EventLoop error in DMX-Worker: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        });

        Ok(Self {
            socket,
            universe_cache: HashMap::new(),
            storage: ConfigStorage::load_or_new(),
            last_storage_load: Instant::now(),
            mqtt_client: client,
        })
    }

    pub async fn listen(&mut self) {
        let mut buf = [0u8; 1144];
        log::info!(
            "sACN Unicast Receiver listening on port {}",
            self.socket.local_addr().unwrap().port()
        );

        loop {
            // Hot-reload configuration every 5 seconds
            if self.last_storage_load.elapsed() > Duration::from_secs(5) {
                self.storage = ConfigStorage::load_or_new();
                self.last_storage_load = Instant::now();
            }

            match self.socket.recv_from(&mut buf) {
                Ok((size, _addr)) => self.handle_udp_packet(&buf[..size]).await,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
                Err(e) => log::error!("Error receiving UDP packet: {}", e),
            }
        }
    }

    async fn handle_udp_packet(&mut self, data: &[u8]) {
        let (universe, dmx_data) = match parse_sacn(data) {
            Some(parsed) => parsed,
            None => return,
        };

        let old_universe_data = self.universe_cache.get(&universe).cloned();
        self.universe_cache.insert(universe, dmx_data.to_vec());

        for device in self.storage.dev_mapping.values() {
            if self.device_needs_update(device, universe, dmx_data, old_universe_data.as_deref()) {
                self.send_device_dmx_update(device, universe, dmx_data)
                    .await;
            }
        }
    }

    fn device_needs_update(
        &self,
        device: &UpdateConfig,
        current_universe: u16,
        current_dmx: &[u8],
        old_dmx: Option<&[u8]>,
    ) -> bool {
        for cfg in &device.dmx_config {
            if cfg.universe as u16 == current_universe {
                if has_config_changed(cfg, current_dmx, old_dmx) {
                    return true;
                }
            }
        }
        false
    }

    async fn send_device_dmx_update(
        &self,
        device: &UpdateConfig,
        trigger_universe: u16,
        trigger_dmx: &[u8],
    ) {
        let mut set_dmx_msg = SetDmx {
            outputs: Vec::new(),
        };

        for cfg in &device.dmx_config {
            let source = if cfg.universe as u16 == trigger_universe {
                Some(trigger_dmx)
            } else {
                self.universe_cache
                    .get(&(cfg.universe as u16))
                    .map(|v| v.as_slice())
            };

            if let Some(data) = source {
                if let Some(dmx_values) = extract_dmx_range(cfg, data) {
                    log::debug!("Updated DMX values: {:?}", dmx_values);
                    set_dmx_msg.outputs.push(OutputData {
                        dmx_start_addr: cfg.start_addr as u32,
                        dmx_values,
                    });
                }
            }
        }

        if !set_dmx_msg.outputs.is_empty() {
            let topic = format!("showtime/set_dmx/{}", device.mac_addr);
            let payload = set_dmx_msg.encode_to_vec();
            let client = self.mqtt_client.clone();
            let mac = device.mac_addr.clone();

            tokio::spawn(async move {
                if client
                    .publish(topic, QoS::AtMostOnce, false, payload)
                    .await
                    .is_ok()
                {
                    log::debug!("Sent DMX update for device {}", mac);
                }
            });
        }
    }
}

fn parse_sacn(data: &[u8]) -> Option<(u16, &[u8])> {
    if data.len() < 125 || data[0..2] != [0x00, 0x10] || &data[4..16] != b"ASC-E1.17\0\0\0" {
        return None;
    }

    let universe = u16::from_be_bytes([data[113], data[114]]);
    let dmx_start = 126;
    let dmx_len = 512.min(data.len() - dmx_start);
    let dmx_data = &data[dmx_start..dmx_start + dmx_len];

    Some((universe, dmx_data))
}

fn has_config_changed(cfg: &DmxConfig, current_dmx: &[u8], old_dmx: Option<&[u8]>) -> bool {
    let start = (cfg.start_addr as usize).saturating_sub(1);
    let len = if cfg.mode == 1 {
        (cfg.led_count as usize) * 3
    } else {
        3
    };
    let end = (start + len).min(512);

    if start >= current_dmx.len() {
        return false;
    }

    let current_range = &current_dmx[start..end.min(current_dmx.len())];

    match old_dmx {
        Some(old) if start < old.len() => {
            let old_range = &old[start..end.min(old.len())];
            current_range != old_range
        }
        _ => true,
    }
}

fn extract_dmx_range(cfg: &DmxConfig, data: &[u8]) -> Option<Vec<u8>> {
    let start = (cfg.start_addr as usize).saturating_sub(1);
    let len = if cfg.mode == 1 {
        (cfg.led_count as usize) * 3
    } else {
        3
    };
    let end = (start + len).min(512);

    if start < data.len() {
        Some(data[start..end.min(data.len())].to_vec())
    } else {
        None
    }
}

pub async fn run_dmx_loop() {
    let port: u16 = std::env::var("SACN_PORT")
        .unwrap_or_else(|_| "5568".to_string())
        .parse()
        .expect("Invalid SACN_PORT");

    if let Ok(mut receiver) = SacnReceiver::new(port) {
        receiver.listen().await;
    }
}
