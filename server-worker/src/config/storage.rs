use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};

use crate::esp::UpdateConfig;

#[derive(Serialize, Deserialize)]
pub struct ConfigStorage {
    pub dev_mapping: HashMap<String, UpdateConfig>,
}

impl ConfigStorage {
    pub fn load_or_new() -> Self {
        ConfigStorage::load_from_file().unwrap_or(ConfigStorage {
            dev_mapping: HashMap::new(),
        })
    }

    pub fn get_device_config(&self, mac_addr: &String) -> Option<UpdateConfig> {
        self.dev_mapping.get(mac_addr).cloned()
    }

    pub fn set_device_config(&mut self, config: UpdateConfig) {
        self.dev_mapping.insert(config.mac_addr.clone(), config);
        self.save_to_file();
    }

    fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write("device_config.toml", toml_string)?;
        Ok(())
    }

    fn load_from_file() -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string("device_config.toml")?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}
