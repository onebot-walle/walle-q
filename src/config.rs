use rs_qq::client::device::DeviceInfo;
use rs_qq::Config as QQConfig;
use serde::{Deserialize, Serialize};
use std::io::Read;
use walle_core::ImplConfig;

const CONFIG_PATH: &str = "neve.yaml";

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Config {
    pub onebot: ImplConfig,
    pub qq: QQConfig,
}

impl Config {
    pub(crate) fn load_or_new() -> Self {
        Self::load_from_file().unwrap_or_else(|_| {
            let mut config = Self::default();
            config.save_to_file().unwrap();
            config.qq.device_info = DeviceInfo::load_or_new();
            config
        })
    }

    pub(crate) fn load_from_file() -> Result<Self, std::io::Error> {
        let mut file = std::fs::File::open(CONFIG_PATH)?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(serde_yaml::from_str(&buf).unwrap())
    }

    pub(crate) fn save_to_file(&self) -> Result<(), std::io::Error> {
        let config_str = serde_yaml::to_string(&self).unwrap();
        std::fs::write(CONFIG_PATH, config_str)?;
        Ok(())
    }
}
