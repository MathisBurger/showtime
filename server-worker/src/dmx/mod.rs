use std::net::UdpSocket;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use rumqttc::QoS;
use prost::Message;

use crate::config::storage::ConfigStorage;
use crate::esp::{SetDmx, set_dmx::OutputData};
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
        
        // Own MQTT client for DMX updates
        let (client, mut eventloop) = init_mqtt_conn("dmx-worker");

        // We MUST poll the eventloop for the client to work
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
        log::info!("sACN Unicast Receiver listening on port {}", self.socket.local_addr().unwrap().port());

        loop {
            // Reload configuration every 5 seconds from TOML to stay independent
            if self.last_storage_load.elapsed() > Duration::from_secs(5) {
                self.storage = ConfigStorage::load_or_new();
                self.last_storage_load = Instant::now();
            }

            match self.socket.recv_from(&mut buf) {
                Ok((size, _addr)) => {
                    self.process_packet(&buf[..size]).await;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
                Err(e) => {
                    log::error!("Error receiving UDP packet: {}", e);
                }
            }
        }
    }

    async fn process_packet(&mut self, data: &[u8]) {
        // Basic sACN validation
        if data.len() < 125 { 
            log::debug!("Packet too short: {}", data.len());
            return; 
        }
        if data[0..2] != [0x00, 0x10] { return; }
        if &data[4..16] != b"ASC-E1.17\0\0\0" { return; }

        let universe = u16::from_be_bytes([data[113], data[114]]);
        let dmx_data = &data[126..126 + 512.min(data.len() - 126)];
        
        log::info!("Received sACN packet for universe {}", universe);

        // Get old data for this universe for diffing
        let old_universe_data = self.universe_cache.get(&universe).cloned();
        
        // Update cache
        self.universe_cache.insert(universe, dmx_data.to_vec());

        // Iterate through all configured devices
        for device in self.storage.dev_mapping.values() {
            let mut device_requires_update = false;
            
            // Check if any output of this device is affected by the current packet
            for cfg in &device.dmx_config {
                if cfg.universe as u16 == universe {
                    let start = (cfg.start_addr as usize).saturating_sub(1);
                    let len = if cfg.mode == 1 { 
                        (cfg.led_count as usize) * 3 
                    } else { 
                        3 
                    };
                    let end = (start + len).min(512);
                    
                    if start >= dmx_data.len() { continue; }
                    let current_range = &dmx_data[start..end.min(dmx_data.len())];

                    // Compare with old cache to see if we need to send an update
                    if let Some(old_u) = &old_universe_data {
                        if start < old_u.len() {
                            let old_range = &old_u[start..end.min(old_u.len())];
                            if current_range != old_range {
                                device_requires_update = true;
                                break;
                            }
                        } else {
                            device_requires_update = true;
                            break;
                        }
                    } else {
                        // No cache yet for this universe, assume changed
                        device_requires_update = true;
                        break;
                    }
                }
            }

            // If any output changed, send the WHOLE state for ALL outputs of this ESP
            if device_requires_update {
                let mut set_dmx_msg = SetDmx { outputs: Vec::new() };
                
                for cfg in &device.dmx_config {
                    // Try to get data for this output (either from current packet or from cache)
                    let source_data = if cfg.universe as u16 == universe {
                        Some(dmx_data.to_vec())
                    } else {
                        self.universe_cache.get(&(cfg.universe as u16)).cloned()
                    };

                    if let Some(u_data) = source_data {
                        let start = (cfg.start_addr as usize).saturating_sub(1);
                        let len = if cfg.mode == 1 { 
                            (cfg.led_count as usize) * 3 
                        } else { 
                            3 
                        };
                        let end = (start + len).min(512);

                        if start < u_data.len() {
                            set_dmx_msg.outputs.push(OutputData {
                                esp_pin: cfg.esp_pin,
                                dmx_values: u_data[start..end.min(u_data.len())].to_vec(),
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
                        if let Ok(_) = client.publish(topic, QoS::AtMostOnce, false, payload).await {
                            log::info!("Sent DMX update for device {}", mac);
                        }
                    });
                }
            }
        }
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
