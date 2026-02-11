use std::collections::VecDeque;

use crate::{common::get_current_unix, esp::EspStatusMessage};

#[derive(Clone, Copy)]
pub enum DeviceStatus {
    Online,
    Offline,
    Overdue,
}

#[derive(Clone)]
pub struct EspDevice {
    pub device_name: String,
    pub color: (u32, u32, u32),
    pub last_sacn_pkt: u32,
    pub last_status_update: u128,
    pub status: DeviceStatus,
    pub mac_addr: String,
    pub signal_strength: u32,
    pub dmx_universe: u32,
    pub dmx_lower_addr: u32,
    pub dmx_upper_addr: u32,
    pub dmx_configs: Vec<crate::esp::DmxConfig>,
    pub history: VecDeque<EspStatusMessage>,
}

impl EspDevice {
    pub fn new(msg: &EspStatusMessage) -> Self {
        // Extrahiere Werte aus dem ersten DmxConfig (falls vorhanden) für Legacy-Zwecke
        let (dmx_universe, dmx_lower_addr, dmx_upper_addr) = msg
            .dmx_config
            .first()
            .map(|cfg| {
                (
                    cfg.universe,
                    cfg.start_addr,
                    cfg.start_addr + (cfg.led_count * 3).saturating_sub(1),
                )
            })
            .unwrap_or((0, 0, 0));

        EspDevice {
            device_name: msg.device_name.clone(),
            color: (msg.r, msg.g, msg.b),
            last_sacn_pkt: msg.last_sacn_pkt,
            last_status_update: get_current_unix(),
            status: DeviceStatus::Online,
            signal_strength: msg.signal_strength,
            mac_addr: msg.mac_addr.clone(),
            dmx_universe,
            dmx_lower_addr,
            dmx_upper_addr,
            dmx_configs: msg.dmx_config.clone(),
            history: VecDeque::new(),
        }
    }

    pub fn update_with_message(&mut self, msg: &EspStatusMessage) {
        self.device_name = msg.device_name.clone();
        self.color = (msg.r, msg.g, msg.b);
        self.last_sacn_pkt = msg.last_sacn_pkt;
        self.last_status_update = get_current_unix();
        self.signal_strength = msg.signal_strength;

        // Extrahiere Werte aus dem ersten DmxConfig (falls vorhanden)
        if let Some(cfg) = msg.dmx_config.first() {
            self.dmx_universe = cfg.universe;
            self.dmx_lower_addr = cfg.start_addr;
            self.dmx_upper_addr = cfg.start_addr + (cfg.led_count * 3).saturating_sub(1);
        }

        self.dmx_configs = msg.dmx_config.clone();

        self.history.push_back(msg.clone());
        if self.history.len() > 99 {
            self.history.pop_front();
        }
    }

    pub fn update_status(&mut self) {
        let time_diff = get_current_unix() - self.last_status_update;

        if time_diff < 100 {
            self.status = DeviceStatus::Online;
        } else if time_diff < 1000 {
            self.status = DeviceStatus::Overdue;
        } else {
            self.status = DeviceStatus::Offline;
        }
    }
}
