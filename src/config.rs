use std::io::Read;

use rand::SeedableRng;
use ricq::device::Device;
use ricq::version::{get_version, Protocol};
use ricq::Config as RsQQConfig;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use walle_core::config::ImplConfig;

use crate::WALLE_Q;

type IOResult<T> = Result<T, std::io::Error>;

const CONFIG_PATH: &str = "walle-q.yaml";
const DEVICE_PATH: &str = "device.json";

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Config {
    #[serde(flatten)]
    pub command: crate::command::Comm,
    pub onebot: ImplConfig,
    #[serde(flatten)]
    pub qq: QQConfig,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct QQConfig {
    pub(crate) uin: Option<u64>,
    pub(crate) password: Option<String>,
    pub(crate) protocol: Option<u8>,
    pub(crate) str_protocol: Option<String>,
}

impl QQConfig {
    fn get_protocol(&self) -> Protocol {
        if let Some(protocol) = self.protocol {
            protocol.try_into().unwrap()
        } else if let Some(str_protocol) = &self.str_protocol {
            str_protocol.as_str().try_into().unwrap()
        } else {
            Protocol::IPad
        }
    }
}

trait NewConfig: Sized {
    fn new_config() -> Self;
    fn ser(&self) -> IOResult<String>;
    fn de(s: &str) -> IOResult<Self>;
}

trait LoadConfig: for<'de> Deserialize<'de> + Serialize + NewConfig {
    fn save_to_file(&self, path: &str) -> IOResult<()> {
        let data = self.ser()?;
        std::fs::write(path, data)
    }

    fn load_from_file(path: &str) -> IOResult<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        Self::de(&data)
    }

    fn load_or_new(path: &str) -> IOResult<Self> {
        info!(target: WALLE_Q, "loading {}", path);
        match Self::load_from_file(path) {
            Ok(config) => {
                info!(target: WALLE_Q, "success load from {}", path);
                Ok(config)
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::Other => {
                    warn!("Serialize error: {}", e);
                    Err(e)
                }
                _ => {
                    warn!("open {} failed: {}", path, e);
                    info!(target: WALLE_Q, "creating new {}", path);
                    let config = Self::new_config();
                    config.save_to_file(path)?;
                    Ok(config)
                }
            },
        }
    }
}

impl NewConfig for Config {
    fn new_config() -> Self {
        Self::default()
    }

    fn ser(&self) -> IOResult<String> {
        serde_yaml::to_string(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn de(s: &str) -> IOResult<Self> {
        serde_yaml::from_str(s).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl LoadConfig for Config {}

impl NewConfig for Device {
    fn new_config() -> Self {
        Self::random()
    }

    fn ser(&self) -> IOResult<String> {
        serde_json::to_string(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    fn de(s: &str) -> IOResult<Self> {
        serde_json::from_str(s).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }
}

impl LoadConfig for Device {}

impl Config {
    pub(crate) fn load() -> Result<Self, std::io::Error> {
        Self::load_or_new(CONFIG_PATH)
    }
}

pub(crate) fn load_device(con: &QQConfig) -> IOResult<RsQQConfig> {
    Ok(RsQQConfig {
        device: Device::load_or_new(DEVICE_PATH).unwrap_or_else(|_| {
            Device::random_with_rng(&mut rand::prelude::StdRng::seed_from_u64(
                con.uin.unwrap_or_default(),
            ))
        }),
        version: get_version(con.get_protocol()),
    })
}
